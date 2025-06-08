//! Signal routing module

pub mod local;
pub mod distributed;

pub use local::{SignalRouter, RoutingTable};
pub use distributed::{DistributedRouter, DistributedConfig, RoutingInfo};