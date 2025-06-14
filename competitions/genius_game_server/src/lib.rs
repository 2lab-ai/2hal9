pub mod server;
pub mod games;
pub mod collective;
pub mod sota;
pub mod analytics;
pub mod streaming;
pub mod ai_providers;

pub use server::GeniusGameServer;
pub use games::{Game, GameEngine, GameResult};
pub use collective::CollectiveIntelligence;
pub use sota::SOTAManager;

#[cfg(test)]
mod tests {
    // Module-level unit tests would go here
}