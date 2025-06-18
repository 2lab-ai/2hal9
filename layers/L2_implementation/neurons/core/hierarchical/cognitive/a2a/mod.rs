//! Agent-to-Agent (A2A) Protocol Module
//! 
//! Implements direct communication between cognitive layers as independent agents.
//! Based on the L9 philosophy from the HAL9 meeting where each layer can:
//! - Operate independently
//! - Use different frameworks
//! - Communicate directly without central coordination
//! - Self-organize and emerge

pub mod protocol;
pub mod direct_connection;
pub mod emergence_detector;
pub mod self_reorganization;
pub mod true_self_organization;

pub use protocol::{A2AProtocol, A2AAgent, AgentMessage, AgentCapability, L9PhilosophyAgent};
pub use direct_connection::{DirectNeuralConnection, DirectNeuralNetwork, DiscoveryMessage};
pub use emergence_detector::{EmergenceDetector, EmergenceReport, EmergenceEvent as DetectorEmergenceEvent, EmergenceType};
pub use self_reorganization::{SelfReorganizingNetwork, ReorganizationEvent, ReorganizationReport};
pub use true_self_organization::{
    TrueSelfOrganizingNetwork, PrimordialNeuron, EmergenceEvent, EmergenceReport as SelfOrgEmergenceReport
};