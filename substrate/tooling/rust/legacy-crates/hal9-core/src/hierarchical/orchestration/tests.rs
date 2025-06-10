//! Comprehensive unit tests for orchestration layer modules

use super::*;
use crate::hierarchical::cognitive::CognitiveLayer;
use std::collections::HashMap;
use uuid::Uuid;

/// Test utilities for orchestration
mod test_utils {
    use super::*;
    
    pub fn create_test_unit_descriptor(layer: CognitiveLayer) -> UnitDescriptor {
        UnitDescriptor {
            id: Uuid::new_v4(),
            unit_type: UnitType::Neuron,
            layer,
            capabilities: vec![
                Capability {
                    name: "processing".to_string(),
                    version: "1.0".to_string(),
                    performance: 0.8,
                },
            ],
            resource_requirements: ResourceRequirements {
                cpu_cores: 1.0,
                memory_mb: 512,
                bandwidth_mbps: 10.0,
            },
        }
    }
    
    pub fn create_test_connection() -> Connection {
        Connection {
            connection_type: ConnectionType::Forward,
            weight: 1.0,
            latency_ms: 5.0,
            bandwidth_limit: Some(100.0),
            properties: HashMap::new(),
        }
    }
    
    pub fn create_test_signal() -> OrchestrationSignal {
        OrchestrationSignal {
            id: Uuid::new_v4(),
            source: Uuid::new_v4(),
            signal_type: SignalType::Data,
            priority: 0.5,
            payload: serde_json::json!({"data": "test"}),
            routing_hints: RoutingHints {
                preferred_path: None,
                avoid_units: vec![],
                max_hops: None,
                deadline: None,
            },
        }
    }
}

mod topology_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_topology_snapshot() {
        let unit1 = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        let unit2 = create_test_unit_descriptor(CognitiveLayer::Implementation);
        let unit3 = create_test_unit_descriptor(CognitiveLayer::Operational);
        
        let mut units = HashMap::new();
        units.insert(unit1.id, unit1.clone());
        units.insert(unit2.id, unit2.clone());
        units.insert(unit3.id, unit3.clone());
        
        let connections = vec![
            (unit1.id, unit2.id, create_test_connection()),
            (unit2.id, unit3.id, create_test_connection()),
        ];
        
        let snapshot = TopologySnapshot {
            timestamp: chrono::Utc::now(),
            units,
            connections,
            metrics: TopologyMetrics {
                total_units: 3,
                total_connections: 2,
                average_degree: 1.33,
                clustering_coefficient: 0.0,
                diameter: 2,
            },
        };
        
        assert_eq!(snapshot.units.len(), 3);
        assert_eq!(snapshot.connections.len(), 2);
        assert_eq!(snapshot.metrics.total_units, 3);
    }
    
    #[tokio::test]
    async fn test_topology_metrics() {
        let metrics = TopologyMetrics {
            total_units: 10,
            total_connections: 20,
            average_degree: 4.0,
            clustering_coefficient: 0.3,
            diameter: 5,
        };
        
        assert_eq!(metrics.total_units, 10);
        assert_eq!(metrics.average_degree, 4.0);
        assert!(metrics.clustering_coefficient < 1.0);
    }
}

mod routing_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_orchestration_signal() {
        let signal = create_test_signal();
        
        assert!(!signal.id.is_nil());
        assert!(!signal.source.is_nil());
        assert!(matches!(signal.signal_type, SignalType::Data));
        assert_eq!(signal.priority, 0.5);
    }
    
    #[tokio::test]
    async fn test_routing_hints() {
        let mut hints = RoutingHints {
            preferred_path: Some(vec![Uuid::new_v4(), Uuid::new_v4()]),
            avoid_units: vec![Uuid::new_v4()],
            max_hops: Some(5),
            deadline: Some(chrono::Utc::now() + chrono::Duration::seconds(60)),
        };
        
        assert!(hints.preferred_path.is_some());
        assert_eq!(hints.avoid_units.len(), 1);
        assert_eq!(hints.max_hops, Some(5));
        
        // Test modification
        hints.max_hops = Some(10);
        assert_eq!(hints.max_hops, Some(10));
    }
}

mod coordination_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_consensus_proposal() {
        let proposal = ConsensusProposal {
            proposal_id: Uuid::new_v4(),
            proposer: Uuid::new_v4(),
            value: serde_json::json!({"action": "update"}),
            timeout: std::time::Duration::from_secs(60),
            required_votes: 3,
        };
        
        assert!(!proposal.proposal_id.is_nil());
        assert!(!proposal.proposer.is_nil());
        assert_eq!(proposal.timeout.as_secs(), 60);
    }
    
    #[tokio::test]
    async fn test_distributed_state() {
        let state = DistributedState {
            state_id: Uuid::new_v4(),
            version: 1,
            data: HashMap::from([
                ("key1".to_string(), serde_json::json!("value1")),
                ("key2".to_string(), serde_json::json!(42)),
            ]),
            metadata: StateMetadata {
                owner: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                ttl: Some(std::time::Duration::from_secs(3600)),
                replication_factor: 3,
                temporal_scale: None,
            },
        };
        
        assert!(!state.state_id.is_nil());
        assert_eq!(state.version, 1);
        assert_eq!(state.data.len(), 2);
    }
    
    #[tokio::test]
    async fn test_state_metadata() {
        let metadata = StateMetadata {
            owner: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            ttl: Some(std::time::Duration::from_secs(3600)),
            replication_factor: 3,
            temporal_scale: None,
        };
        
        assert!(!metadata.owner.is_nil());
        assert_eq!(metadata.replication_factor, 3);
        assert_eq!(metadata.ttl.unwrap().as_secs(), 3600);
    }
}

mod flow_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_forward_signal() {
        let signal = ForwardSignal {
            signal_id: Uuid::new_v4(),
            source: Uuid::new_v4(),
            content: serde_json::json!({"data": "test"}),
            urgency: 0.8,
            constraints: RoutingConstraints {
                max_latency_ms: Some(100.0),
                required_capabilities: vec!["processing".to_string()],
                avoid_units: vec![],
                prefer_units: vec![],
            },
        };
        
        assert!(!signal.signal_id.is_nil());
        assert!(!signal.source.is_nil());
        assert_eq!(signal.urgency, 0.8);
        assert_eq!(signal.constraints.max_latency_ms, Some(100.0));
    }
    
    #[tokio::test]
    async fn test_routing_decision() {
        let decision = RoutingDecision {
            targets: vec![
                RoutingTarget {
                    unit_id: Uuid::new_v4(),
                    weight: 0.7,
                    priority: 0.9,
                },
                RoutingTarget {
                    unit_id: Uuid::new_v4(),
                    weight: 0.3,
                    priority: 0.5,
                },
            ],
            strategy: RoutingStrategy::LoadBalanced,
            estimated_latency_ms: 25.0,
        };
        
        assert_eq!(decision.targets.len(), 2);
        assert_eq!(decision.targets[0].weight, 0.7);
        assert!(matches!(decision.strategy, RoutingStrategy::LoadBalanced));
        assert_eq!(decision.estimated_latency_ms, 25.0);
    }
}

mod optimization_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_optimization_report() {
        let report = OptimizationReport {
            changes_made: vec![
                TopologyChange::UnitAdded { id: Uuid::new_v4() },
                TopologyChange::ConnectionAdded {
                    from: Uuid::new_v4(),
                    to: Uuid::new_v4(),
                },
            ],
            performance_improvement: 0.15,
            resource_savings: ResourceSavings {
                cpu_cores_saved: 2.0,
                memory_mb_saved: 1024,
                bandwidth_mbps_saved: 50.0,
            },
        };
        
        assert_eq!(report.changes_made.len(), 2);
        assert_eq!(report.performance_improvement, 0.15);
        assert_eq!(report.resource_savings.memory_mb_saved, 1024);
    }
    
    #[tokio::test]
    async fn test_topology_changes() {
        let changes = vec![
            TopologyChange::UnitRemoved { id: Uuid::new_v4() },
            TopologyChange::ConnectionWeightChanged {
                from: Uuid::new_v4(),
                to: Uuid::new_v4(),
                old: 1.0,
                new: 1.5,
            },
        ];
        
        assert_eq!(changes.len(), 2);
        if let TopologyChange::ConnectionWeightChanged { old, new, .. } = &changes[1] {
            assert_eq!(*old, 1.0);
            assert_eq!(*new, 1.5);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::test_utils::*;
    
    // Mock implementations for integration testing
    struct MockTopologyManager;
    struct MockFlowController;
    struct MockStateCoordinator;
    struct MockSignalRouter;
    
    #[async_trait]
    impl TopologyManager for MockTopologyManager {
        async fn add_node(&mut self, _descriptor: UnitDescriptor) -> Result<NodeId> {
            Ok(Uuid::new_v4())
        }
        
        async fn remove_node(&mut self, _node_id: NodeId) -> Result<()> {
            Ok(())
        }
        
        async fn add_edge(&mut self, _from: NodeId, _to: NodeId, _connection: Connection) -> Result<()> {
            Ok(())
        }
        
        async fn remove_edge(&mut self, _from: NodeId, _to: NodeId) -> Result<()> {
            Ok(())
        }
        
        async fn get_node(&self, _node_id: NodeId) -> Result<Option<&UnitDescriptor>> {
            Ok(None)
        }
        
        async fn get_neighbors(&self, _node_id: NodeId) -> Result<Vec<(NodeId, &Connection)>> {
            Ok(vec![])
        }
        
        async fn shortest_path(&self, _from: NodeId, _to: NodeId) -> Result<Option<Vec<NodeId>>> {
            Ok(None)
        }
        
        async fn metrics(&self) -> Result<TopologyMetrics> {
            Ok(TopologyMetrics {
                total_units: 0,
                total_connections: 0,
                average_degree: 0.0,
                clustering_coefficient: 0.0,
                diameter: 0,
            })
        }
        
        async fn evolve(&mut self, _current_fitness: f32) -> Result<()> {
            Ok(())
        }
        
        async fn calculate_fitness(&self) -> Result<f32> {
            Ok(0.8)
        }
    }
    
    #[async_trait]
    impl FlowController for MockFlowController {
        async fn route_forward(&self, _signal: ForwardSignal) -> Result<RoutingDecision> {
            Ok(RoutingDecision {
                targets: vec![],
                strategy: RoutingStrategy::ShortestPath,
                estimated_latency_ms: 10.0,
            })
        }
        
        async fn route_backward(&self, _gradient: BackwardGradient) -> Result<RoutingDecision> {
            Ok(RoutingDecision {
                targets: vec![],
                strategy: RoutingStrategy::ShortestPath,
                estimated_latency_ms: 10.0,
            })
        }
        
        async fn balance_load(&mut self) -> Result<LoadBalanceReport> {
            Ok(LoadBalanceReport {
                rebalanced_units: 0,
                moved_connections: 0,
                load_variance_before: 0.1,
                load_variance_after: 0.05,
            })
        }
        
        async fn metrics(&self) -> Result<FlowMetrics> {
            Ok(FlowMetrics {
                total_signals_routed: 100,
                average_hops: 2.5,
                average_latency_ms: 15.0,
                congestion_points: vec![],
                throughput_per_second: 1000.0,
            })
        }
        
        async fn update_weights(&mut self, _performance: &PerformanceMetrics) -> Result<()> {
            Ok(())
        }
    }
    
    #[async_trait]
    impl StateCoordinator for MockStateCoordinator {
        async fn synchronize(&self, _state: DistributedState) -> Result<SyncResult> {
            Ok(SyncResult {
                synchronized_units: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
                conflicts: vec![],
                version: 1,
            })
        }
        
        async fn consensus(&self, proposal: ConsensusProposal) -> Result<ConsensusResult> {
            Ok(ConsensusResult {
                accepted: true,
                value: proposal.value,
                votes: vec![],
                duration: std::time::Duration::from_secs(1),
            })
        }
        
        async fn lock(&self, _resource: ResourceId) -> Result<DistributedLock> {
            // Return mock lock - actual implementation would handle this differently
            unimplemented!("Mock lock not fully implemented")
        }
        
        async fn snapshot(&self) -> Result<GlobalStateSnapshot> {
            Ok(GlobalStateSnapshot {
                timestamp: chrono::Utc::now(),
                units: HashMap::new(),
                global_variables: HashMap::new(),
                consistency_level: ConsistencyLevel::Strong,
            })
        }
        
        async fn subscribe(&self, _filter: StateFilter) -> Result<StateSubscription> {
            // Return mock subscription - actual implementation would handle this differently  
            unimplemented!("Mock subscription not fully implemented")
        }
    }
    
    #[async_trait]
    impl SignalRouter for MockSignalRouter {
        async fn route(&self, _signal: &RoutableSignal) -> Result<Vec<RoutingPath>> {
            Ok(vec![RoutingPath {
                path: vec![Uuid::new_v4()],
                total_latency_ms: 10.0,
                min_bandwidth_mbps: 100.0,
                reliability: 0.99,
                cost: 1.0,
            }])
        }
        
        async fn update_topology(&mut self, _change: routing::TopologyChange) -> Result<()> {
            Ok(())
        }
        
        async fn statistics(&self) -> Result<RoutingStatistics> {
            Ok(RoutingStatistics {
                total_routed: 100,
                failed_routes: 0,
                average_path_length: 2.5,
                average_latency_ms: 15.0,
                cache_hit_rate: 0.8,
            })
        }
        
        async fn optimize(&mut self) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_full_orchestration_stack() {
        let mut orchestrator = DefaultOrchestrator::new(
            Box::new(MockTopologyManager),
            Box::new(MockFlowController),
            Box::new(MockStateCoordinator),
            Box::new(MockSignalRouter),
        );
        
        orchestrator.initialize().await.unwrap();
        
        // Add units
        let unit1 = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        let unit2 = create_test_unit_descriptor(CognitiveLayer::Implementation);
        
        let id1 = orchestrator.add_unit(unit1).await.unwrap();
        let id2 = orchestrator.add_unit(unit2).await.unwrap();
        
        // Connect units
        orchestrator.connect(id1, id2, create_test_connection()).await.unwrap();
        
        // Test signal routing
        let signal = create_test_signal();
        let path = orchestrator.route(signal).await.unwrap();
        assert!(!path.is_empty());
        
        // Optimize topology
        let report = orchestrator.optimize().await.unwrap();
        assert!(report.performance_improvement >= 0.0);
        
        // Get topology
        let topology = orchestrator.topology().await.unwrap();
        assert!(topology.timestamp <= chrono::Utc::now());
    }
}

/// Performance benchmarks
#[cfg(test)]
mod benchmarks {
    use super::*;
    use super::test_utils::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_signal_creation() {
        let iterations = 10000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = create_test_signal();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() as f64 / iterations as f64;
        
        println!("Average signal creation time: {:.2} μs", avg_time);
        assert!(avg_time < 10.0); // Should be under 10 microseconds
    }
    
    #[tokio::test]
    async fn benchmark_topology_metrics() {
        let mut units = HashMap::new();
        let mut connections = vec![];
        
        // Create a large topology
        for i in 0..100 {
            let unit = create_test_unit_descriptor(CognitiveLayer::Implementation);
            units.insert(unit.id, unit.clone());
            
            if i > 0 {
                connections.push((
                    units.values().nth(i - 1).unwrap().id,
                    unit.id,
                    create_test_connection(),
                ));
            }
        }
        
        let start = Instant::now();
        
        let _snapshot = TopologySnapshot {
            timestamp: chrono::Utc::now(),
            units,
            connections,
            metrics: TopologyMetrics {
                total_units: 100,
                total_connections: 99,
                average_degree: 1.98,
                clustering_coefficient: 0.0,
                diameter: 99,
            },
        };
        
        let elapsed = start.elapsed();
        println!("Topology snapshot creation time: {:.2} ms", elapsed.as_millis());
        assert!(elapsed.as_millis() < 100); // Should be under 100ms
    }
}