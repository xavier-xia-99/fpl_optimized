//! Player entity types

use crate::types::{Cost, Position, TeamCode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique player identifier from FPL API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

impl PlayerId {
    /// Create a new player ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for PlayerId {
    fn from(id: u32) -> Self {
        PlayerId(id)
    }
}

impl From<PlayerId> for u32 {
    fn from(id: PlayerId) -> Self {
        id.0
    }
}

/// A Fantasy Premier League player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique identifier
    pub id: PlayerId,

    /// Display name (e.g., "Salah")
    pub web_name: String,

    /// Full name (first + second)
    pub full_name: Option<String>,

    /// Team code
    pub team_code: TeamCode,

    /// Playing position
    pub position: Position,

    /// Current price in tenths of millions
    pub cost: Cost,

    /// Ownership percentage (0.0 - 100.0)
    pub ownership_percent: f64,

    // Optional statistics
    /// Recent form
    pub form: Option<f64>,

    /// ICT index
    pub ict_index: Option<f64>,

    /// Expected points this week
    pub ep_this: Option<f64>,

    /// Points per game
    pub points_per_game: Option<f64>,

    /// Bonus points system score
    pub bps: Option<f64>,

    /// Influence stat
    pub influence: Option<f64>,

    /// Creativity stat
    pub creativity: Option<f64>,

    /// Threat stat
    pub threat: Option<f64>,
}

impl Player {
    /// Create a new player with minimal required fields
    pub fn new(
        id: u32,
        web_name: impl Into<String>,
        team_code: u8,
        position: Position,
        cost: u16,
    ) -> Self {
        Self {
            id: PlayerId(id),
            web_name: web_name.into(),
            full_name: None,
            team_code: TeamCode(team_code),
            position,
            cost: Cost(cost),
            ownership_percent: 0.0,
            form: None,
            ict_index: None,
            ep_this: None,
            points_per_game: None,
            bps: None,
            influence: None,
            creativity: None,
            threat: None,
        }
    }

    /// Create a minimal player for testing
    #[cfg(test)]
    pub fn test(id: u32, position: Position, cost: u16, team: u8) -> Self {
        Self {
            id: PlayerId(id),
            web_name: format!("Player{}", id),
            full_name: None,
            team_code: TeamCode(team),
            position,
            cost: Cost(cost),
            ownership_percent: 5.0,
            form: None,
            ict_index: None,
            ep_this: None,
            points_per_game: None,
            bps: None,
            influence: None,
            creativity: None,
            threat: None,
        }
    }

    /// Check if this player is a differential (low ownership)
    pub fn is_differential(&self, threshold: f64) -> bool {
        self.ownership_percent <= threshold
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {})",
            self.web_name,
            self.position.short_name(),
            self.cost
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::test(1, Position::Midfielder, 127, 14);
        assert_eq!(player.id, PlayerId(1));
        assert_eq!(player.position, Position::Midfielder);
        assert_eq!(player.cost, Cost(127));
    }

    #[test]
    fn test_player_display() {
        let player = Player::new(1, "Salah", 14, Position::Midfielder, 127);
        assert_eq!(format!("{}", player), "Salah (MID, £12.7M)");
    }

    #[test]
    fn test_is_differential() {
        let mut player = Player::test(1, Position::Midfielder, 100, 14);
        player.ownership_percent = 3.0;
        assert!(player.is_differential(5.0));
        
        player.ownership_percent = 10.0;
        assert!(!player.is_differential(5.0));
    }

    #[test]
    fn test_player_id_from() {
        let id: PlayerId = 42u32.into();
        assert_eq!(id.raw(), 42);
    }
}
