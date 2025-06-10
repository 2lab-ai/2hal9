//! Hierarchical Architecture - Foundation for HAL9's transformation
//!
//! This module implements the hierarchical abstraction architecture that transforms
//! HAL9 from a flat neuron network into a deeply layered system with emergent intelligence.
//!
//! ## Architecture Layers
//!
//! 1. **Substrate Layer** - Foundation (Runtime, Transport, Storage, Resources)
//! 2. **Protocol Layer** - Communication (Messages, Negotiation, Versioning, Streams)
//! 3. **Cognitive Layer** - Processing (Neurons, Learning, Patterns)
//! 4. **Orchestration Layer** - Coordination (Topology, Flow, State, Routing)
//! 5. **Intelligence Layer** - Emergence (Meta-learning, Self-organization, Creativity)
//!
//! Each layer provides abstractions to the layers above it, enabling complex behaviors
//! to emerge from simple rules.

pub mod cognitive;
pub mod intelligence;
pub mod interfaces;
pub mod orchestration;
pub mod protocol;
pub mod substrate;

pub use interfaces::*;

/// Version of the hierarchical architecture
pub const ARCHITECTURE_VERSION: &str = "1.0.0";

/// Re-export commonly used types
pub mod prelude {
    pub use crate::hierarchical::{
        cognitive::{CognitiveLayer, CognitiveUnit},
        intelligence::{Goal, IntelligenceCoordinator},
        interfaces::{LayerBoundary, LayerInterface},
        orchestration::{Orchestrator, TopologyManager},
        protocol::{Protocol, ProtocolVersion},
        substrate::{Substrate, SubstrateCapabilities},
    };
}
