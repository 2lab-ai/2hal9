//! Comprehensive tests for the Protocol Layer

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::hierarchical::substrate::transport::{ChannelTransport, TypedTransport};
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};
    use uuid::Uuid;

    /// Test protocol version compatibility
    #[test]
    fn test_protocol_version_compatibility() {
        let v1_0_0 = ProtocolVersion::new(1, 0, 0);
        let v1_0_1 = ProtocolVersion::new(1, 0, 1);
        let v1_1_0 = ProtocolVersion::new(1, 1, 0);
        let v2_0_0 = ProtocolVersion::new(2, 0, 0);
        
        // Same major version = compatible
        assert!(v1_0_0.is_compatible_with(&v1_0_1));
        assert!(v1_0_0.is_compatible_with(&v1_1_0));
        
        // Different major version = incompatible
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    }

    /// Test protocol negotiation
    #[tokio::test]
    async fn test_protocol_negotiation() {
        // Create two negotiators representing different peers
        let protocols_a = vec![
            ProtocolDescriptor {
                id: "test-protocol".to_string(),
                versions: vec![ProtocolVersion::new(1, 0, 0), ProtocolVersion::new(1, 1, 0)],
                features: vec!["feature1".to_string(), "feature2".to_string()],
            }
        ];
        
        let capabilities_a = ProtocolCapabilities {
            compression: vec![CompressionType::None, CompressionType::Gzip, CompressionType::Zstd],
            encryption: vec![EncryptionType::None, EncryptionType::Tls],
            max_message_size: 1_000_000,
            streaming: true,
            bidirectional: true,
            ordered_delivery: true,
        };
        
        let negotiator_a = DefaultNegotiator::new(protocols_a.clone(), capabilities_a.clone());
        
        // Peer B with slightly different capabilities
        let protocols_b = [ProtocolDescriptor {
                id: "test-protocol".to_string(),
                versions: vec![ProtocolVersion::new(1, 0, 0)],
                features: vec!["feature1".to_string(), "feature3".to_string()],
            }];
        
        let capabilities_b = ProtocolCapabilities {
            compression: vec![CompressionType::None, CompressionType::Gzip],
            encryption: vec![EncryptionType::None],
            max_message_size: 500_000,
            streaming: true,
            bidirectional: false,
            ordered_delivery: true,
        };
        
        // A initiates to B
        let offer = ProtocolOffer {
            protocols: protocols_a,
            capabilities: capabilities_a,
            preferences: NegotiationPreferences {
                preferred_compression: Some(CompressionType::Zstd),
                preferred_encryption: Some(EncryptionType::Tls),
                require_encryption: false,
                require_ordered_delivery: true,
            },
        };
        
        let response = negotiator_a.initiate(&offer).await.unwrap();
        
        // Should select compatible protocol and parameters
        assert_eq!(response.selected_protocol.id, "test-protocol");
        assert_eq!(response.negotiated_params.version, ProtocolVersion::new(1, 0, 0));
    }

    /// Test signal protocol end-to-end
    #[tokio::test]
    async fn test_signal_protocol_e2e() {
        let transport = Arc::new(ChannelTransport::new());
        let protocol = SignalProtocol::new(transport.clone());
        
        // Create two neurons
        let neuron1 = Uuid::new_v4();
        let neuron2 = Uuid::new_v4();
        
        // Set up receiver
        let mut receiver = protocol.receive_signals(neuron2).await.unwrap();
        
        // Send signal from neuron1 to neuron2
        let signal = SignalMessage {
            id: Uuid::new_v4(),
            source_neuron: neuron1,
            target_neuron: Some(neuron2),
            timestamp: chrono::Utc::now(),
            activation: Activation::new("Test signal".to_string(), 0.85),
            metadata: serde_json::json!({"test": true}),
        };
        
        protocol.send_signal(signal.clone()).await.unwrap();
        
        // Receive and verify
        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout")
            .expect("No signal received");
        
        assert_eq!(received.id, signal.id);
        assert_eq!(received.activation.content, "Test signal");
        assert_eq!(received.activation.strength, 0.85);
    }

    /// Test gradient protocol with accumulation
    #[tokio::test]
    async fn test_gradient_protocol_accumulation() {
        let transport = Arc::new(ChannelTransport::new());
        let protocol = GradientProtocol::new(transport.clone(), 3); // batch size 3
        
        let neuron_id = Uuid::new_v4();
        
        // Set up receiver
        let mut receiver = protocol.receive_gradients(neuron_id).await.unwrap();
        
        // Accumulate gradients (should auto-flush at 3)
        for i in 0..3 {
            let grad = Gradient::new(0.1 * (i + 1) as f32, vec![1.0, 0.0, -1.0]);
            protocol.accumulate_gradient(neuron_id, grad).await.unwrap();
        }
        
        // Should receive accumulated gradient
        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout")
            .expect("No gradient received");
        
        // Verify accumulated values (averaged)
        assert_eq!(received.gradient.error, 0.2); // (0.1 + 0.2 + 0.3) / 3
        assert_eq!(received.gradient.accumulated_steps, 2); // 0 + 1 + 1
    }

    /// Test consensus protocol with voting
    #[tokio::test]
    async fn test_consensus_protocol_voting() {
        let transport = Arc::new(ChannelTransport::new());
        
        // Create 5 nodes for consensus
        let mut nodes = Vec::new();
        let mut protocols = Vec::new();
        
        for _ in 0..5 {
            let node_id = Uuid::new_v4();
            nodes.push(node_id);
            
            let protocol = ConsensusProtocol::new(
                transport.clone(),
                node_id,
                consensus::ConsensusAlgorithm::SimpleMajority,
            );
            protocols.push(protocol);
        }
        
        // Add all nodes as participants
        for protocol in &protocols {
            for &node in &nodes {
                protocol.add_participant(node).await.unwrap();
            }
        }
        
        // Start receivers
        for protocol in &protocols {
            protocol.start_receiver().await.unwrap();
        }
        
        // Node 0 proposes a value
        let proposal_value = serde_json::json!({
            "action": "test_consensus",
            "value": 42
        });
        
        let proposal_id = protocols[0].propose(
            proposal_value.clone(),
            Duration::from_secs(60)
        ).await.unwrap();
        
        // Wait for proposal to propagate
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // 3 nodes vote Accept (including proposer)
        for i in 0..3 {
            protocols[i].vote(proposal_id, consensus::Vote::Accept).await.unwrap();
        }
        
        // 1 node votes Reject
        protocols[3].vote(proposal_id, consensus::Vote::Reject).await.unwrap();
        
        // 1 node abstains
        
        // Wait for consensus
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Check consensus reached (3/5 = 60% > 50% required)
        let metrics = protocols[0].metrics();
        assert_eq!(metrics.consensus_reached, 1);
        assert_eq!(metrics.proposals_created, 1);
    }

    /// Test protocol manager integration
    #[tokio::test]
    async fn test_protocol_manager() {
        let transport = Arc::new(ChannelTransport::new());
        let config = ProtocolManagerConfig {
            negotiation_timeout: Duration::from_secs(5),
            enable_compression: true,
            enable_encryption: false,
            max_message_size: 1_000_000,
            version_migration_enabled: true,
        };
        
        let manager = ProtocolManager::new(config, transport);
        
        // Initialize standard protocols
        manager.initialize_protocols().await.unwrap();
        
        // Verify protocols are registered
        assert!(manager.get_protocol("signal-protocol").is_some());
        assert!(manager.get_protocol("gradient-protocol").is_some());
        assert!(manager.get_protocol("consensus-protocol").is_some());
        
        // Test versioned message handling
        let test_message = b"Hello, Protocol Layer!";
        let versioned = VersionedMessage::new(
            "test-protocol",
            ProtocolVersion::new(1, 0, 0),
            test_message.to_vec(),
        );
        
        let encoded = serde_json::to_vec(&versioned).unwrap();
        let (protocol_id, decoded) = manager.receive_versioned_message(&encoded).await.unwrap();
        
        assert_eq!(protocol_id, "test-protocol");
        assert_eq!(decoded, test_message);
    }

    /// Test message compression
    #[test]
    fn test_compression_types() {
        let data = b"This is some test data that should be compressed. ".repeat(100);
        
        // Test Gzip
        {
            use flate2::write::GzEncoder;
            use flate2::read::GzDecoder;
            use flate2::Compression;
            use std::io::{Write, Read};
            
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data).unwrap();
            let compressed = encoder.finish().unwrap();
            
            assert!(compressed.len() < data.len());
            
            let mut decoder = GzDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();
            
            assert_eq!(decompressed, data);
        }
        
        // Test LZ4
        {
            let compressed = lz4_flex::compress_prepend_size(&data);
            assert!(compressed.len() < data.len());
            
            let decompressed = lz4_flex::decompress_size_prepended(&compressed).unwrap();
            assert_eq!(decompressed, data);
        }
        
        // Test Zstd
        {
            let compressed = zstd::encode_all(&data[..], 3).unwrap();
            assert!(compressed.len() < data.len());
            
            let decompressed = zstd::decode_all(&compressed[..]).unwrap();
            assert_eq!(decompressed, data);
        }
    }

    /// Test stream protocol
    #[tokio::test]
    async fn test_stream_protocol() {
        use streams::*;
        
        let config = StreamConfig {
            stream_id: Uuid::new_v4(),
            buffer_size: 10,
            backpressure: BackpressureStrategy::Buffer,
            ordering: StreamOrdering::Ordered,
            reliability: StreamReliability::AtLeastOnce,
        };
        
        let (mut stream1, mut stream2) = ChannelStream::new(config.buffer_size);
        
        // Send data from stream1 to stream2
        let data = b"Stream data chunk".to_vec();
        stream1.send(data.clone()).await.unwrap();
        
        // Receive on stream2
        let received = stream2.recv().await.unwrap().unwrap();
        assert_eq!(received, data);
        
        // Test stream closing
        assert!(!stream1.is_closed());
        Box::new(stream1).close().await.unwrap();
        assert!(stream2.is_closed());
    }

    /// Test protocol metrics
    #[tokio::test]
    async fn test_protocol_metrics() {
        let transport = Arc::new(ChannelTransport::new());
        
        // Test signal protocol metrics
        {
            let protocol = SignalProtocol::new(transport.clone());
            
            // Send some signals
            for i in 0..5 {
                let signal = SignalMessage {
                    id: Uuid::new_v4(),
                    source_neuron: Uuid::new_v4(),
                    target_neuron: Some(Uuid::new_v4()),
                    timestamp: chrono::Utc::now(),
                    activation: Activation::new(format!("Signal {}", i), 0.5),
                    metadata: serde_json::json!({}),
                };
                protocol.send_signal(signal).await.unwrap();
            }
            
            let metrics = protocol.metrics();
            assert_eq!(metrics.signals_sent, 5);
            assert_eq!(metrics.efficiency, 1.0); // No drops
        }
        
        // Test gradient protocol metrics
        {
            let protocol = GradientProtocol::new(transport.clone(), 10);
            
            // Send some gradients
            for i in 0..3 {
                let msg = GradientMessage {
                    id: Uuid::new_v4(),
                    source_neuron: Uuid::new_v4(),
                    target_neuron: Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    gradient: Gradient::new(0.1 * i as f32, vec![1.0, 2.0, 3.0]),
                    learning_context: gradient::LearningContext {
                        learning_rate: 0.01,
                        momentum: 0.9,
                        batch_size: 32,
                        epoch: i,
                        loss_type: gradient::LossType::MeanSquaredError,
                    },
                };
                protocol.send_gradient(msg).await.unwrap();
            }
            
            let metrics = protocol.metrics();
            assert_eq!(metrics.gradients_sent, 3);
            assert!(metrics.average_error > 0.0);
        }
    }

    /// Test version migration
    #[test]
    fn test_version_migration() {
        use versioning::*;
        
        let mut registry = VersionRegistry::new(ProtocolVersion::new(1, 1, 0));
        
        // Register a migration
        registry.register_migration(Box::new(V1_0ToV1_1Migration::default()));
        
        // Test migration
        let old_message = serde_json::json!({
            "existing_field": "value"
        });
        let old_bytes = serde_json::to_vec(&old_message).unwrap();
        
        let migrated = registry.migrate_to_current(
            &ProtocolVersion::new(1, 0, 0),
            &old_bytes
        ).unwrap();
        
        let migrated_value: serde_json::Value = serde_json::from_slice(&migrated).unwrap();
        
        // Should have added new field
        assert_eq!(migrated_value["existing_field"], "value");
        assert_eq!(migrated_value["new_field"], "default");
    }
}