// E2E Test Suite for HAL9 Server
// This module provides comprehensive end-to-end testing

pub mod test_framework;

#[cfg(test)]
pub mod scenarios {
    pub mod full_lifecycle_test;
    pub mod performance_test;
    pub mod auth_test;
    pub mod websocket_test;
}

// Re-export framework components for easy use
pub use test_framework::{
    E2ETestClient, TestConfig, WebSocketTestClient, TestServer,
    Assertions, Fixtures, PerfTest
};