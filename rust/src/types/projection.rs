//! Expected points projections for players

use crate::types::{Gameweek, PlayerId, TeamCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expected points and minutes for a player in a specific gameweek
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameweekProjection {
    /// Player ID
    pub player_id: PlayerId,

    /// Gameweek number
    pub gameweek: Gameweek,

    /// Expected points for this gameweek
    pub expected_points: f64,

    /// Expected minutes for this gameweek
    pub expected_minutes: f64,

    /// Opponent team code (if available)
    pub opponent_team: Option<TeamCode>,

    /// Whether playing at home
    pub is_home: Option<bool>,

    /// Raw expected points (points_md * 90 / xmins)
    pub raw_xp: Option<f64>,
}

impl PlayerGameweekProjection {
    /// Create a new projection
    pub fn new(player_id: PlayerId, gameweek: Gameweek, expected_points: f64) -> Self {
        Self {
            player_id,
            gameweek,
            expected_points,
            expected_minutes: 90.0,
            opponent_team: None,
            is_home: None,
            raw_xp: None,
        }
    }

    /// Create with full details
    pub fn with_details(
        player_id: PlayerId,
        gameweek: Gameweek,
        expected_points: f64,
        expected_minutes: f64,
        opponent_team: Option<TeamCode>,
        is_home: Option<bool>,
    ) -> Self {
        let raw_xp = if expected_minutes > 0.0 {
            Some(expected_points * 90.0 / expected_minutes.max(1.0))
        } else {
            None
        };
        Self {
            player_id,
            gameweek,
            expected_points,
            expected_minutes,
            opponent_team,
            is_home,
            raw_xp,
        }
    }
}

/// Collection of all projections for optimization
#[derive(Debug, Clone, Default)]
pub struct ProjectionData {
    /// Indexed by (player_id, gameweek) for O(1) lookup
    projections: HashMap<(PlayerId, Gameweek), PlayerGameweekProjection>,

    /// Gameweeks covered
    gameweeks: Vec<Gameweek>,
}

impl ProjectionData {
    /// Create a new empty projection data collection
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
            gameweeks: Vec::new(),
        }
    }

    /// Add a projection
    pub fn insert(&mut self, projection: PlayerGameweekProjection) {
        let key = (projection.player_id, projection.gameweek);
        
        // Track unique gameweeks
        if !self.gameweeks.contains(&projection.gameweek) {
            self.gameweeks.push(projection.gameweek);
            self.gameweeks.sort();
        }
        
        self.projections.insert(key, projection);
    }

    /// Get a projection for a player and gameweek
    pub fn get(&self, player_id: PlayerId, gw: Gameweek) -> Option<&PlayerGameweekProjection> {
        self.projections.get(&(player_id, gw))
    }

    /// Get expected points for a player in a gameweek
    pub fn get_expected_points(&self, player_id: PlayerId, gw: Gameweek) -> f64 {
        self.projections
            .get(&(player_id, gw))
            .map(|p| p.expected_points)
            .unwrap_or(0.0)
    }

    /// Get all gameweeks
    pub fn gameweeks(&self) -> &[Gameweek] {
        &self.gameweeks
    }

    /// Get the first (next) gameweek
    pub fn first_gameweek(&self) -> Option<Gameweek> {
        self.gameweeks.first().copied()
    }

    /// Get total expected points for a player across all gameweeks
    pub fn total_expected_points(&self, player_id: PlayerId) -> f64 {
        self.gameweeks
            .iter()
            .map(|&gw| self.get_expected_points(player_id, gw))
            .sum()
    }

    /// Get expected points for a player in the first gameweek
    pub fn next_gw_expected_points(&self, player_id: PlayerId) -> f64 {
        self.first_gameweek()
            .map(|gw| self.get_expected_points(player_id, gw))
            .unwrap_or(0.0)
    }

    /// Get number of projections
    pub fn len(&self) -> usize {
        self.projections.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }

    /// Iterate over all projections
    pub fn iter(&self) -> impl Iterator<Item = &PlayerGameweekProjection> {
        self.projections.values()
    }

    /// Get maximum expected points across all projections
    pub fn max_expected_points(&self) -> f64 {
        self.projections
            .values()
            .map(|p| p.expected_points)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Get maximum expected minutes across all projections
    pub fn max_expected_minutes(&self) -> f64 {
        self.projections
            .values()
            .map(|p| p.expected_minutes)
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_creation() {
        let proj = PlayerGameweekProjection::new(PlayerId(1), Gameweek(21), 5.5);
        assert_eq!(proj.player_id, PlayerId(1));
        assert_eq!(proj.expected_points, 5.5);
    }

    #[test]
    fn test_projection_data_insert_and_get() {
        let mut data = ProjectionData::new();
        let proj = PlayerGameweekProjection::new(PlayerId(1), Gameweek(21), 5.5);
        data.insert(proj);

        assert_eq!(data.get_expected_points(PlayerId(1), Gameweek(21)), 5.5);
        assert_eq!(data.get_expected_points(PlayerId(1), Gameweek(22)), 0.0);
    }

    #[test]
    fn test_projection_data_gameweeks() {
        let mut data = ProjectionData::new();
        data.insert(PlayerGameweekProjection::new(PlayerId(1), Gameweek(21), 5.0));
        data.insert(PlayerGameweekProjection::new(PlayerId(2), Gameweek(22), 4.0));
        data.insert(PlayerGameweekProjection::new(PlayerId(3), Gameweek(21), 6.0));

        assert_eq!(data.gameweeks(), &[Gameweek(21), Gameweek(22)]);
        assert_eq!(data.first_gameweek(), Some(Gameweek(21)));
    }

    #[test]
    fn test_total_expected_points() {
        let mut data = ProjectionData::new();
        data.insert(PlayerGameweekProjection::new(PlayerId(1), Gameweek(21), 5.0));
        data.insert(PlayerGameweekProjection::new(PlayerId(1), Gameweek(22), 4.0));
        data.insert(PlayerGameweekProjection::new(PlayerId(1), Gameweek(23), 6.0));

        assert_eq!(data.total_expected_points(PlayerId(1)), 15.0);
    }
}
