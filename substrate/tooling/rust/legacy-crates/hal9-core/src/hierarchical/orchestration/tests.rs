//! Comprehensive unit tests for orchestration layer modules

use super::*;
use crate::hierarchical::cognitive::CognitiveLayer;
use std::sync::Arc;
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
            target: None,
            content: SignalContent::Data("test data".to_string()),
            priority: Priority::Normal,
            ttl: 10,
            metadata: HashMap::new(),
        }
    }
    
    pub fn create_test_flow_config() -> FlowConfiguration {
        FlowConfiguration {
            flow_type: FlowType::Sequential,
            stages: vec![
                FlowStage {
                    name: "input".to_string(),
                    units: vec![Uuid::new_v4()],
                    processing_mode: ProcessingMode::Parallel,
                },
                FlowStage {
                    name: "processing".to_string(),
                    units: vec![Uuid::new_v4(), Uuid::new_v4()],
                    processing_mode: ProcessingMode::Sequential,
                },
                FlowStage {
                    name: "output".to_string(),
                    units: vec![Uuid::new_v4()],
                    processing_mode: ProcessingMode::Parallel,
                },
            ],
            error_handling: ErrorHandling::Retry { max_attempts: 3 },
        }
    }
}

mod topology_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_topology_builder() {
        let mut builder = TopologyBuilder::new();
        
        // Add nodes
        let node1 = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        let node2 = create_test_unit_descriptor(CognitiveLayer::Implementation);
        let node3 = create_test_unit_descriptor(CognitiveLayer::Operational);
        
        let id1 = builder.add_node(node1.clone()).await.unwrap();
        let id2 = builder.add_node(node2.clone()).await.unwrap();
        let id3 = builder.add_node(node3.clone()).await.unwrap();
        
        // Add connections
        builder.connect(id1, id2, create_test_connection()).await.unwrap();
        builder.connect(id2, id3, create_test_connection()).await.unwrap();
        
        // Build topology
        let topology = builder.build().await.unwrap();
        
        assert_eq!(topology.nodes.len(), 3);
        assert_eq!(topology.edges.len(), 2);
        assert!(topology.validate().is_ok());
    }
    
    #[tokio::test]
    async fn test_topology_traversal() {
        let mut manager = TopologyManager::new();
        
        // Create a simple graph
        let nodes = vec![
            create_test_unit_descriptor(CognitiveLayer::Reflexive),
            create_test_unit_descriptor(CognitiveLayer::Implementation),
            create_test_unit_descriptor(CognitiveLayer::Operational),
        ];
        
        let mut ids = vec![];
        for node in nodes {
            ids.push(manager.add_node(node).await.unwrap());
        }
        
        // Create linear connections
        manager.connect(ids[0], ids[1], create_test_connection()).await.unwrap();
        manager.connect(ids[1], ids[2], create_test_connection()).await.unwrap();
        
        // Test traversal
        let path = manager.find_path(ids[0], ids[2]).await.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], ids[0]);
        assert_eq!(path[1], ids[1]);
        assert_eq!(path[2], ids[2]);
    }
    
    #[tokio::test]
    async fn test_topology_cycles() {
        let mut manager = TopologyManager::new();
        
        // Create nodes
        let ids: Vec<_> = (0..3).map(|i| {
            let node = create_test_unit_descriptor(match i {
                0 => CognitiveLayer::Reflexive,
                1 => CognitiveLayer::Implementation,
                _ => CognitiveLayer::Operational,
            });
            manager.add_node(node).boxed()
        }).collect();
        
        let ids: Vec<Uuid> = futures::future::join_all(ids).await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        
        // Create cycle
        manager.connect(ids[0], ids[1], create_test_connection()).await.unwrap();
        manager.connect(ids[1], ids[2], create_test_connection()).await.unwrap();
        manager.connect(ids[2], ids[0], create_test_connection()).await.unwrap();
        
        // Detect cycles
        let has_cycle = manager.has_cycles().await.unwrap();
        assert!(has_cycle);
    }
    
    #[tokio::test]
    async fn test_topology_optimization() {
        let mut optimizer = TopologyOptimizer::new();
        
        // Create suboptimal topology
        let mut topology = Topology::new();
        
        // Add nodes in a chain
        let nodes: Vec<_> = (0..5).map(|_| {
            create_test_unit_descriptor(CognitiveLayer::Implementation)
        }).collect();
        
        let mut ids = vec![];
        for node in nodes {
            ids.push(topology.add_node(node));
        }
        
        // Create inefficient connections
        for i in 0..4 {
            for j in (i+1)..5 {
                topology.add_edge(ids[i], ids[j], create_test_connection());
            }
        }
        
        // Optimize
        let optimized = optimizer.optimize(topology).await.unwrap();
        
        // Should have fewer edges after optimization
        assert!(optimized.edges.len() < 10); // Was fully connected (10 edges)
    }
}

mod routing_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_basic_routing() {
        let mut router = BasicRouter::new();
        
        // Setup simple topology
        let unit1 = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        let unit2 = create_test_unit_descriptor(CognitiveLayer::Implementation);
        
        router.register_unit(unit1.clone()).await.unwrap();
        router.register_unit(unit2.clone()).await.unwrap();
        router.add_route(unit1.id, unit2.id, RouteOptions::default()).await.unwrap();
        
        // Route signal
        let mut signal = create_test_signal();
        signal.source = unit1.id;
        signal.target = Some(unit2.id);
        
        let path = router.route(signal).await.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], unit1.id);
        assert_eq!(path[1], unit2.id);
    }
    
    #[tokio::test]
    async fn test_load_balanced_routing() {
        let mut router = LoadBalancedRouter::new();
        
        // Create multiple paths
        let source = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        let targets = vec![
            create_test_unit_descriptor(CognitiveLayer::Implementation),
            create_test_unit_descriptor(CognitiveLayer::Implementation),
            create_test_unit_descriptor(CognitiveLayer::Implementation),
        ];
        
        router.register_unit(source.clone()).await.unwrap();
        for target in &targets {
            router.register_unit(target.clone()).await.unwrap();
            router.add_route(source.id, target.id, RouteOptions::default()).await.unwrap();
        }
        
        // Route multiple signals and verify distribution
        let mut distribution = HashMap::new();
        for _ in 0..100 {
            let mut signal = create_test_signal();
            signal.source = source.id;
            
            let path = router.route(signal).await.unwrap();
            let target = path.last().unwrap();
            *distribution.entry(*target).or_insert(0) += 1;
        }
        
        // Verify relatively even distribution
        for count in distribution.values() {
            assert!(*count > 20 && *count < 50); // Roughly 33% each
        }
    }
    
    #[tokio::test]
    async fn test_priority_routing() {
        let mut router = PriorityRouter::new();
        
        // Setup units with different capacities
        let source = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        let high_priority_target = create_test_unit_descriptor(CognitiveLayer::Operational);
        let low_priority_target = create_test_unit_descriptor(CognitiveLayer::Implementation);
        
        router.register_unit(source.clone()).await.unwrap();
        router.register_unit(high_priority_target.clone()).await.unwrap();
        router.register_unit(low_priority_target.clone()).await.unwrap();
        
        // Add routes with different priorities
        router.add_route(source.id, high_priority_target.id, 
            RouteOptions { priority: Priority::High, ..Default::default() }).await.unwrap();
        router.add_route(source.id, low_priority_target.id,
            RouteOptions { priority: Priority::Low, ..Default::default() }).await.unwrap();
        
        // Route high priority signal
        let mut signal = create_test_signal();
        signal.source = source.id;
        signal.priority = Priority::High;
        
        let path = router.route(signal).await.unwrap();
        assert_eq!(path.last().unwrap(), &high_priority_target.id);
    }
    
    #[tokio::test]
    async fn test_broadcast_routing() {
        let mut router = BroadcastRouter::new();
        
        // Setup broadcast topology
        let source = create_test_unit_descriptor(CognitiveLayer::Strategic);
        let targets = vec![
            create_test_unit_descriptor(CognitiveLayer::Tactical),
            create_test_unit_descriptor(CognitiveLayer::Tactical),
            create_test_unit_descriptor(CognitiveLayer::Operational),
        ];
        
        router.register_unit(source.clone()).await.unwrap();
        for target in &targets {
            router.register_unit(target.clone()).await.unwrap();
            router.add_route(source.id, target.id, RouteOptions::default()).await.unwrap();
        }
        
        // Broadcast signal
        let mut signal = create_test_signal();
        signal.source = source.id;
        signal.target = None; // Broadcast
        
        let paths = router.broadcast(signal).await.unwrap();
        assert_eq!(paths.len(), 3);
        
        // Verify all targets received
        let target_ids: HashSet<_> = targets.iter().map(|t| t.id).collect();
        let reached_ids: HashSet<_> = paths.iter().map(|p| *p.last().unwrap()).collect();
        assert_eq!(target_ids, reached_ids);
    }
}

mod coordination_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_coordinator_initialization() {
        let mut coordinator = DistributedCoordinator::new();
        
        // Initialize with units
        let units = vec![
            create_test_unit_descriptor(CognitiveLayer::Reflexive),
            create_test_unit_descriptor(CognitiveLayer::Implementation),
            create_test_unit_descriptor(CognitiveLayer::Operational),
        ];
        
        for unit in units {
            coordinator.register_unit(unit).await.unwrap();
        }
        
        // Start coordination
        coordinator.start().await.unwrap();
        
        let status = coordinator.status().await.unwrap();
        assert_eq!(status.registered_units, 3);
        assert!(status.is_running);
    }
    
    #[tokio::test]
    async fn test_consensus_mechanism() {
        let mut consensus = ConsensusManager::new();
        
        // Add participants
        let participants = vec![
            Participant { id: Uuid::new_v4(), weight: 1.0 },
            Participant { id: Uuid::new_v4(), weight: 1.0 },
            Participant { id: Uuid::new_v4(), weight: 1.0 },
        ];
        
        for p in &participants {
            consensus.add_participant(p.clone()).await.unwrap();
        }
        
        // Propose value
        let proposal = Proposal {
            id: Uuid::new_v4(),
            value: serde_json::json!({"action": "update_parameter", "value": 0.5}),
            proposer: participants[0].id,
        };
        
        consensus.propose(proposal.clone()).await.unwrap();
        
        // Vote
        consensus.vote(proposal.id, participants[0].id, Vote::Accept).await.unwrap();
        consensus.vote(proposal.id, participants[1].id, Vote::Accept).await.unwrap();
        consensus.vote(proposal.id, participants[2].id, Vote::Reject).await.unwrap();
        
        // Check consensus (2/3 accepted)
        let result = consensus.check_consensus(proposal.id).await.unwrap();
        assert!(result.is_accepted);
    }
    
    #[tokio::test]
    async fn test_distributed_locking() {
        let lock_manager = DistributedLockManager::new();
        
        // Acquire lock
        let resource_id = Uuid::new_v4();
        let holder1 = Uuid::new_v4();
        let holder2 = Uuid::new_v4();
        
        let lock1 = lock_manager.acquire(resource_id, holder1, Duration::from_secs(5)).await.unwrap();
        assert!(lock1.is_acquired);
        
        // Try to acquire same lock
        let lock2 = lock_manager.try_acquire(resource_id, holder2).await.unwrap();
        assert!(!lock2.is_acquired);
        
        // Release and re-acquire
        lock_manager.release(resource_id, holder1).await.unwrap();
        let lock3 = lock_manager.acquire(resource_id, holder2, Duration::from_secs(5)).await.unwrap();
        assert!(lock3.is_acquired);
    }
    
    #[tokio::test]
    async fn test_event_propagation() {
        let mut event_bus = EventBus::new();
        
        // Subscribe units
        let sub1 = Uuid::new_v4();
        let sub2 = Uuid::new_v4();
        let sub3 = Uuid::new_v4();
        
        event_bus.subscribe(sub1, EventType::StateChange).await.unwrap();
        event_bus.subscribe(sub2, EventType::StateChange).await.unwrap();
        event_bus.subscribe(sub3, EventType::PerformanceUpdate).await.unwrap();
        
        // Publish events
        let event1 = Event {
            id: Uuid::new_v4(),
            event_type: EventType::StateChange,
            source: Uuid::new_v4(),
            data: serde_json::json!({"state": "active"}),
            timestamp: chrono::Utc::now(),
        };
        
        let event2 = Event {
            id: Uuid::new_v4(),
            event_type: EventType::PerformanceUpdate,
            source: Uuid::new_v4(),
            data: serde_json::json!({"throughput": 1000}),
            timestamp: chrono::Utc::now(),
        };
        
        let recipients1 = event_bus.publish(event1).await.unwrap();
        let recipients2 = event_bus.publish(event2).await.unwrap();
        
        assert_eq!(recipients1.len(), 2); // sub1 and sub2
        assert_eq!(recipients2.len(), 1); // sub3
    }
}

mod flow_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_flow_builder() {
        let mut builder = FlowBuilder::new();
        
        // Build sequential flow
        builder.add_stage("input", vec![
            create_test_unit_descriptor(CognitiveLayer::Reflexive),
        ]).await.unwrap();
        
        builder.add_stage("processing", vec![
            create_test_unit_descriptor(CognitiveLayer::Implementation),
            create_test_unit_descriptor(CognitiveLayer::Implementation),
        ]).await.unwrap();
        
        builder.add_stage("output", vec![
            create_test_unit_descriptor(CognitiveLayer::Operational),
        ]).await.unwrap();
        
        let flow = builder.build().await.unwrap();
        assert_eq!(flow.stages.len(), 3);
        assert!(flow.validate().is_ok());
    }
    
    #[tokio::test]
    async fn test_flow_execution() {
        let mut executor = FlowExecutor::new();
        
        // Create test flow
        let flow = create_test_flow_config();
        executor.register_flow("test_flow", flow).await.unwrap();
        
        // Execute flow
        let input = FlowInput {
            flow_name: "test_flow".to_string(),
            data: serde_json::json!({"value": 42}),
            context: HashMap::new(),
        };
        
        let result = executor.execute(input).await.unwrap();
        assert!(result.success);
        assert!(!result.stage_results.is_empty());
    }
    
    #[tokio::test]
    async fn test_parallel_flow_execution() {
        let mut executor = FlowExecutor::new();
        
        // Create parallel flow
        let flow = FlowConfiguration {
            flow_type: FlowType::Parallel,
            stages: vec![
                FlowStage {
                    name: "parallel_processing".to_string(),
                    units: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
                    processing_mode: ProcessingMode::Parallel,
                },
            ],
            error_handling: ErrorHandling::ContinueOnError,
        };
        
        executor.register_flow("parallel_flow", flow).await.unwrap();
        
        // Execute and measure time
        let start = std::time::Instant::now();
        let input = FlowInput {
            flow_name: "parallel_flow".to_string(),
            data: serde_json::json!({"values": [1, 2, 3]}),
            context: HashMap::new(),
        };
        
        let result = executor.execute(input).await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(result.success);
        // Parallel execution should be faster than sequential
        assert!(elapsed.as_millis() < 300); // Assuming each unit takes ~100ms
    }
    
    #[tokio::test]
    async fn test_flow_error_handling() {
        let mut executor = FlowExecutor::new();
        
        // Create flow with error handling
        let flow = FlowConfiguration {
            flow_type: FlowType::Sequential,
            stages: vec![
                FlowStage {
                    name: "may_fail".to_string(),
                    units: vec![Uuid::new_v4()],
                    processing_mode: ProcessingMode::Sequential,
                },
            ],
            error_handling: ErrorHandling::Retry { max_attempts: 3 },
        };
        
        executor.register_flow("retry_flow", flow).await.unwrap();
        
        // Simulate failing unit
        executor.set_unit_behavior(Uuid::new_v4(), UnitBehavior::FailTwiceThenSucceed).await.unwrap();
        
        let input = FlowInput {
            flow_name: "retry_flow".to_string(),
            data: serde_json::json!({}),
            context: HashMap::new(),
        };
        
        let result = executor.execute(input).await.unwrap();
        assert!(result.success); // Should succeed after retries
        assert_eq!(result.retry_count, 2);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_full_orchestration_stack() {
        // Create orchestrator
        let mut orchestrator = DefaultOrchestrator::new();
        orchestrator.initialize().await.unwrap();
        
        // Build hierarchical topology
        let layers = vec![
            (CognitiveLayer::Reflexive, 3),
            (CognitiveLayer::Implementation, 2),
            (CognitiveLayer::Operational, 2),
            (CognitiveLayer::Tactical, 1),
            (CognitiveLayer::Strategic, 1),
        ];
        
        let mut unit_ids = HashMap::new();
        
        for (layer, count) in layers {
            let mut layer_ids = vec![];
            for _ in 0..count {
                let unit = create_test_unit_descriptor(layer);
                let id = orchestrator.add_unit(unit).await.unwrap();
                layer_ids.push(id);
            }
            unit_ids.insert(layer, layer_ids);
        }
        
        // Connect layers hierarchically
        for reflexive_id in &unit_ids[&CognitiveLayer::Reflexive] {
            for impl_id in &unit_ids[&CognitiveLayer::Implementation] {
                orchestrator.connect(*reflexive_id, *impl_id, create_test_connection()).await.unwrap();
            }
        }
        
        for impl_id in &unit_ids[&CognitiveLayer::Implementation] {
            for op_id in &unit_ids[&CognitiveLayer::Operational] {
                orchestrator.connect(*impl_id, *op_id, create_test_connection()).await.unwrap();
            }
        }
        
        for op_id in &unit_ids[&CognitiveLayer::Operational] {
            orchestrator.connect(*op_id, unit_ids[&CognitiveLayer::Tactical][0], create_test_connection()).await.unwrap();
        }
        
        orchestrator.connect(
            unit_ids[&CognitiveLayer::Tactical][0],
            unit_ids[&CognitiveLayer::Strategic][0],
            create_test_connection()
        ).await.unwrap();
        
        // Test signal routing through hierarchy
        let signal = OrchestrationSignal {
            id: Uuid::new_v4(),
            source: unit_ids[&CognitiveLayer::Reflexive][0],
            target: Some(unit_ids[&CognitiveLayer::Strategic][0]),
            content: SignalContent::Data("hierarchical signal".to_string()),
            priority: Priority::Normal,
            ttl: 10,
            metadata: HashMap::new(),
        };
        
        let path = orchestrator.route(signal).await.unwrap();
        assert!(path.len() >= 5); // Through all layers
        
        // Optimize topology
        let report = orchestrator.optimize().await.unwrap();
        assert!(report.improvements.len() > 0);
        
        // Verify optimized topology
        let topology = orchestrator.topology().await.unwrap();
        assert!(topology.metrics.average_path_length < 5.0);
    }
}

/// Performance benchmarks
#[cfg(test)]
mod benchmarks {
    use super::*;
    use super::test_utils::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_routing_performance() {
        let mut router = LoadBalancedRouter::new();
        
        // Create large topology
        let source = create_test_unit_descriptor(CognitiveLayer::Reflexive);
        router.register_unit(source.clone()).await.unwrap();
        
        let mut targets = vec![];
        for i in 0..100 {
            let target = UnitDescriptor {
                id: Uuid::new_v4(),
                unit_type: UnitType::Neuron,
                layer: CognitiveLayer::Implementation,
                capabilities: vec![],
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1.0,
                    memory_mb: 256,
                    bandwidth_mbps: 10.0,
                },
            };
            router.register_unit(target.clone()).await.unwrap();
            router.add_route(source.id, target.id, RouteOptions::default()).await.unwrap();
            targets.push(target);
        }
        
        // Benchmark routing
        let iterations = 10000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let mut signal = create_test_signal();
            signal.source = source.id;
            let _ = router.route(signal).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() as f64 / iterations as f64;
        
        println!("Average routing time: {:.2} μs", avg_time);
        assert!(avg_time < 50.0); // Should route in under 50 microseconds
    }
    
    #[tokio::test]
    async fn benchmark_topology_optimization() {
        let mut optimizer = TopologyOptimizer::new();
        
        // Create complex topology
        let mut topology = Topology::new();
        
        // Add 50 nodes
        let mut ids = vec![];
        for _ in 0..50 {
            let node = create_test_unit_descriptor(CognitiveLayer::Implementation);
            ids.push(topology.add_node(node));
        }
        
        // Add random connections
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for _ in 0..200 {
            let from = ids[rng.gen_range(0..ids.len())];
            let to = ids[rng.gen_range(0..ids.len())];
            if from != to {
                topology.add_edge(from, to, create_test_connection());
            }
        }
        
        // Benchmark optimization
        let start = Instant::now();
        let optimized = optimizer.optimize(topology).await.unwrap();
        let elapsed = start.elapsed();
        
        println!("Topology optimization time: {:.2} ms", elapsed.as_millis());
        println!("Edges before: 200, after: {}", optimized.edges.len());
        assert!(elapsed.as_millis() < 1000); // Should complete within 1 second
    }
}