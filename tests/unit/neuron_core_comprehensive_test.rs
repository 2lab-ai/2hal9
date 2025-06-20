//! Comprehensive unit tests for neuron core functionality
//! Target: 80% coverage for neuron module

use hal9_core::{Neuron, Layer, Signal};
use hal9_core::neuron::{NeuronState, NeuronMetrics};
use hal9_core::signal::SignalType;
use hal9_core::hierarchical::orchestration::RoutingHints;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod neuron_core_tests {
    use super::*;

    /// Test helper to create a test neuron
    fn create_test_neuron(layer: Layer) -> Arc<RwLock<dyn Neuron>> {
        struct TestNeuron {
            id: Uuid,
            layer: Layer,
            state: NeuronState,
            connections: Vec<Uuid>,
            processed_signals: Vec<Signal>,
        }

        #[async_trait::async_trait]
        impl Neuron for TestNeuron {
            fn id(&self) -> Uuid {
                self.id
            }

            fn layer(&self) -> Layer {
                self.layer
            }

            fn state(&self) -> NeuronState {
                self.state.clone()
            }

            async fn process_signal(&mut self, signal: Signal) -> hal9_core::Result<Vec<Signal>> {
                self.processed_signals.push(signal.clone());
                
                // Simulate processing based on layer
                let output = match self.layer {
                    Layer::L1 => vec![Signal {
                        id: Uuid::new_v4(),
                        source: self.id,
                        signal_type: SignalType::Activation(1.0),
                        payload: signal.payload,
                        timestamp: Utc::now(),
                        routing_hints: Default::default(),
                    }],
                    Layer::L2 => {
                        // L2 compresses information
                        let compressed_payload = signal.payload.len() / 2;
                        vec![Signal {
                            id: Uuid::new_v4(),
                            source: self.id,
                            signal_type: SignalType::Activation(0.8),
                            payload: vec![0u8; compressed_payload],
                            timestamp: Utc::now(),
                            routing_hints: Default::default(),
                        }]
                    }
                    _ => vec![],
                };
                
                Ok(output)
            }

            async fn connect(&mut self, other: Uuid) -> hal9_core::Result<()> {
                if !self.connections.contains(&other) {
                    self.connections.push(other);
                }
                Ok(())
            }

            async fn disconnect(&mut self, other: Uuid) -> hal9_core::Result<()> {
                self.connections.retain(|&id| id != other);
                Ok(())
            }

            fn connections(&self) -> Vec<Uuid> {
                self.connections.clone()
            }

            async fn update_state(&mut self, new_state: NeuronState) -> hal9_core::Result<()> {
                self.state = new_state;
                Ok(())
            }

            fn metrics(&self) -> NeuronMetrics {
                NeuronMetrics {
                    signals_processed: self.processed_signals.len() as u64,
                    avg_processing_time_ms: 1.5,
                    error_rate: 0.0,
                    connections_count: self.connections.len(),
                    memory_usage_bytes: 1024,
                    last_active: Utc::now(),
                }
            }
        }

        Arc::new(RwLock::new(TestNeuron {
            id: Uuid::new_v4(),
            layer,
            state: NeuronState::Active,
            connections: Vec::new(),
            processed_signals: Vec::new(),
        }))
    }

    #[tokio::test]
    async fn test_neuron_creation_all_layers() {
        // Test neuron creation for all layers
        let layers = vec![
            Layer::L1, Layer::L2, Layer::L3, Layer::L4,
            Layer::L5, Layer::L6, Layer::L7, Layer::L8, Layer::L9
        ];

        for layer in layers {
            let neuron = create_test_neuron(layer);
            let n = neuron.read().await;
            assert_eq!(n.layer(), layer);
            assert_eq!(n.state(), NeuronState::Active);
            assert!(n.connections().is_empty());
        }
    }

    #[tokio::test]
    async fn test_neuron_state_transitions() {
        let neuron = create_test_neuron(Layer::L2);
        
        // Test all state transitions
        let states = vec![
            NeuronState::Active,
            NeuronState::Inhibited,
            NeuronState::Learning,
            NeuronState::Consolidating,
            NeuronState::Error,
        ];

        for state in states {
            neuron.write().await.update_state(state.clone()).await.unwrap();
            assert_eq!(neuron.read().await.state(), state);
        }
    }

    #[tokio::test]
    async fn test_signal_processing_by_layer() {
        // Test signal processing for each layer
        let test_signal = Signal {
            id: Uuid::new_v4(),
            source: Uuid::new_v4(),
            signal_type: SignalType::Activation(1.0),
            payload: vec![1, 2, 3, 4, 5, 6, 7, 8],
            timestamp: Utc::now(),
            routing_hints: Default::default(),
        };

        // L1: Reflexive - immediate response
        let l1_neuron = create_test_neuron(Layer::L1);
        let l1_output = l1_neuron.write().await.process_signal(test_signal.clone()).await.unwrap();
        assert_eq!(l1_output.len(), 1);
        assert_eq!(l1_output[0].payload.len(), test_signal.payload.len());

        // L2: Implementation - compression
        let l2_neuron = create_test_neuron(Layer::L2);
        let l2_output = l2_neuron.write().await.process_signal(test_signal.clone()).await.unwrap();
        assert_eq!(l2_output.len(), 1);
        assert!(l2_output[0].payload.len() < test_signal.payload.len()); // Compressed
    }

    #[tokio::test]
    async fn test_neuron_connections() {
        let neuron1 = create_test_neuron(Layer::L2);
        let neuron2_id = Uuid::new_v4();
        let neuron3_id = Uuid::new_v4();

        // Test connecting
        neuron1.write().await.connect(neuron2_id).await.unwrap();
        neuron1.write().await.connect(neuron3_id).await.unwrap();
        
        let connections = neuron1.read().await.connections();
        assert_eq!(connections.len(), 2);
        assert!(connections.contains(&neuron2_id));
        assert!(connections.contains(&neuron3_id));

        // Test duplicate connection (should not add)
        neuron1.write().await.connect(neuron2_id).await.unwrap();
        assert_eq!(neuron1.read().await.connections().len(), 2);

        // Test disconnecting
        neuron1.write().await.disconnect(neuron2_id).await.unwrap();
        let connections = neuron1.read().await.connections();
        assert_eq!(connections.len(), 1);
        assert!(!connections.contains(&neuron2_id));
        assert!(connections.contains(&neuron3_id));
    }

    #[tokio::test]
    async fn test_neuron_metrics() {
        let neuron = create_test_neuron(Layer::L3);
        
        // Process multiple signals
        for i in 0..5 {
            let signal = Signal {
                id: Uuid::new_v4(),
                source: Uuid::new_v4(),
                signal_type: SignalType::Activation(i as f32),
                payload: vec![i as u8; 10],
                timestamp: Utc::now(),
                routing_hints: Default::default(),
            };
            neuron.write().await.process_signal(signal).await.unwrap();
        }

        // Connect to other neurons
        for _ in 0..3 {
            neuron.write().await.connect(Uuid::new_v4()).await.unwrap();
        }

        let metrics = neuron.read().await.metrics();
        assert_eq!(metrics.signals_processed, 5);
        assert_eq!(metrics.connections_count, 3);
        assert_eq!(metrics.error_rate, 0.0);
        assert!(metrics.avg_processing_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_concurrent_signal_processing() {
        use tokio::task::JoinSet;
        
        let neuron = create_test_neuron(Layer::L4);
        let neuron_clone = neuron.clone();

        let mut tasks = JoinSet::new();

        // Spawn multiple concurrent signal processing tasks
        for i in 0..10 {
            let n = neuron_clone.clone();
            tasks.spawn(async move {
                let signal = Signal {
                    id: Uuid::new_v4(),
                    source: Uuid::new_v4(),
                    signal_type: SignalType::Activation(i as f32),
                    payload: vec![i as u8; 20],
                    timestamp: Utc::now(),
                    routing_hints: Default::default(),
                };
                n.write().await.process_signal(signal).await
            });
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            results.push(result.unwrap());
        }

        // Verify all succeeded
        assert_eq!(results.len(), 10);
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_layer_validation() {
        // Test layer parsing from string
        assert_eq!("L1".parse::<Layer>().unwrap(), Layer::L1);
        assert_eq!("L9".parse::<Layer>().unwrap(), Layer::L9);
        assert!("L0".parse::<Layer>().is_err());
        assert!("L10".parse::<Layer>().is_err());
        assert!("invalid".parse::<Layer>().is_err());
    }

    #[tokio::test]
    async fn test_signal_routing_hints() {
        let neuron = create_test_neuron(Layer::L5);
        let target_neuron = Uuid::new_v4();
        
        let signal = Signal {
            id: Uuid::new_v4(),
            source: Uuid::new_v4(),
            signal_type: SignalType::Control("test_command".to_string()),
            payload: vec![],
            timestamp: Utc::now(),
            routing_hints: RoutingHints {
                preferred_path: Some(vec![neuron.read().await.id(), target_neuron]),
                avoid_units: vec![],
                max_hops: Some(3),
                deadline: Some(Utc::now() + chrono::Duration::seconds(5)),
            },
        };

        let result = neuron.write().await.process_signal(signal).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_handling() {
        struct ErrorNeuron {
            id: Uuid,
            should_fail: bool,
        }

        #[async_trait::async_trait]
        impl Neuron for ErrorNeuron {
            fn id(&self) -> Uuid {
                self.id
            }

            fn layer(&self) -> Layer {
                Layer::L1
            }

            fn state(&self) -> NeuronState {
                NeuronState::Error
            }

            async fn process_signal(&mut self, _signal: Signal) -> hal9_core::Result<Vec<Signal>> {
                if self.should_fail {
                    Err(hal9_core::Error::ProcessingError("Test error".to_string()))
                } else {
                    Ok(vec![])
                }
            }

            async fn connect(&mut self, _other: Uuid) -> hal9_core::Result<()> {
                if self.should_fail {
                    Err(hal9_core::Error::ConnectionError("Cannot connect".to_string()))
                } else {
                    Ok(())
                }
            }

            async fn disconnect(&mut self, _other: Uuid) -> hal9_core::Result<()> {
                Ok(())
            }

            fn connections(&self) -> Vec<Uuid> {
                vec![]
            }

            async fn update_state(&mut self, _new_state: NeuronState) -> hal9_core::Result<()> {
                Ok(())
            }

            fn metrics(&self) -> NeuronMetrics {
                NeuronMetrics {
                    signals_processed: 0,
                    avg_processing_time_ms: 0.0,
                    error_rate: if self.should_fail { 1.0 } else { 0.0 },
                    connections_count: 0,
                    memory_usage_bytes: 0,
                    last_active: Utc::now(),
                }
            }
        }

        let error_neuron = Arc::new(RwLock::new(ErrorNeuron {
            id: Uuid::new_v4(),
            should_fail: true,
        }));

        // Test signal processing error
        let signal = Signal {
            id: Uuid::new_v4(),
            source: Uuid::new_v4(),
            signal_type: SignalType::Activation(1.0),
            payload: vec![],
            timestamp: Utc::now(),
            routing_hints: Default::default(),
        };

        let result = error_neuron.write().await.process_signal(signal).await;
        assert!(result.is_err());

        // Test connection error
        let connect_result = error_neuron.write().await.connect(Uuid::new_v4()).await;
        assert!(connect_result.is_err());

        // Test metrics show error rate
        assert_eq!(error_neuron.read().await.metrics().error_rate, 1.0);
    }
}