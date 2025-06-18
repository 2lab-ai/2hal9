//! Database models for HAL9

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Re-export from the game models
pub use crate::genius_game::{Game, GamePlayer, GameRound, GameStatus};

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub email_verification_expires: Option<DateTime<Utc>>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Neuron model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Neuron {
    pub id: Uuid,
    pub layer: String,
    pub system_prompt: String,
    pub settings: serde_json::Value,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Signal model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Signal {
    pub id: Uuid,
    pub from_neuron: String,
    pub to_neuron: String,
    pub layer_from: String,
    pub layer_to: String,
    pub propagation_type: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// API Key model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub permissions: serde_json::Value,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Session model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Memory model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Memory {
    pub id: Uuid,
    pub neuron_id: String,
    pub content: String,
    pub importance: f32,
    pub context: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Player response model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerResponse {
    pub id: Uuid,
    pub round_id: Uuid,
    pub player_id: Uuid,
    pub response: String,
    pub response_time_ms: i32,
    pub is_ai_generated: bool,
    pub creativity_score: Option<f32>,
    pub humor_score: Option<f32>,
    pub relevance_score: Option<f32>,
    pub total_score: Option<f32>,
    pub submitted_at: DateTime<Utc>,
}

/// Vote model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: Uuid,
    pub response_id: Uuid,
    pub voter_id: Uuid,
    pub vote_type: String,
    pub created_at: DateTime<Utc>,
}

/// Game chat message
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GameChat {
    pub id: Uuid,
    pub game_id: Uuid,
    pub player_id: Uuid,
    pub message: String,
    pub message_type: String,
    pub created_at: DateTime<Utc>,
}

/// Game statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GameStats {
    pub id: Uuid,
    pub game_id: Uuid,
    pub total_responses: i32,
    pub ai_responses: i32,
    pub human_responses: i32,
    pub avg_response_time_ms: Option<f32>,
    pub avg_creativity_score: Option<f32>,
    pub avg_humor_score: Option<f32>,
    pub avg_relevance_score: Option<f32>,
    pub most_creative_player_id: Option<Uuid>,
    pub most_funny_player_id: Option<Uuid>,
    pub fastest_player_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

/// Achievement model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub points: i32,
    pub category: String,
    pub created_at: DateTime<Utc>,
}

/// Player achievement model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerAchievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub achievement_id: Uuid,
    pub game_id: Option<Uuid>,
    pub earned_at: DateTime<Utc>,
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total_games_played: i32,
    pub total_wins: i32,
    pub total_score: i32,
    pub avg_score_per_game: Option<f32>,
    pub highest_single_game_score: Option<i32>,
    pub total_votes_received: i32,
    pub creativity_votes: i32,
    pub humor_votes: i32,
    pub achievement_points: i32,
    pub rank: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

// DTOs for API responses

/// User profile response (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            is_active: user.is_active,
            email_verified: user.email_verified,
            created_at: user.created_at,
            last_login: user.last_login,
        }
    }
}

/// Create user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Update user request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

/// Password reset request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// Password reset confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}