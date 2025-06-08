//! 2HAL9 MVP Library - Exports for testing

pub use crate::{Signal, MockNeuron};

// Re-export main module
pub mod main {
    pub use crate::*;
}

// Include the actual implementation
include!("main.rs");