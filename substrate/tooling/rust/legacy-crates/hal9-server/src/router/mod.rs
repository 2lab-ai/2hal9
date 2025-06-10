//! Signal routing module

pub mod distributed;
pub mod local;

pub use distributed::{DistributedConfig, DistributedRouter, RoutingInfo};
pub use local::{RoutingTable, SignalRouter};
