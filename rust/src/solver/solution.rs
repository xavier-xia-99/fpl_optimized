//! Solution types for optimization results

use crate::types::{Cost, Gameweek, PlayerId, Position};
use serde::{Deserialize, Serialize};

/// A player selected in the solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedPlayer {
    pub id: PlayerId,
    pub web_name: String,
    pub team_code: u8,
    pub position: Position,
    pub cost: Cost,
    pub expected_points: f64,
    pub ownership_percent: f64,
    pub is_captain: bool,
    pub is_in_lineup: bool,
    pub multiplier: u8,
}

impl SelectedPlayer {
    /// Calculate gameweek points (expected_points × multiplier)
    pub fn gw_points(&self) -> f64 {
        self.expected_points * self.multiplier as f64
    }
}

/// Result of solving an optimization problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemResult {
    /// Problem name
    pub problem_name: String,

    /// All selected players (squad)
    pub squad: Vec<SelectedPlayer>,

    /// Player IDs in lineup
    pub lineup_ids: Vec<PlayerId>,

    /// Player IDs on bench
    pub bench_ids: Vec<PlayerId>,

    /// Captain player ID
    pub captain_id: Option<PlayerId>,

    /// Vice captain player ID (if applicable)
    pub vice_captain_id: Option<PlayerId>,

    /// Objective value (total expected points)
    pub objective_value: f64,

    /// Total cost of squad
    pub total_cost: Cost,

    /// Gameweek this was solved for
    pub gameweek: Option<Gameweek>,
}

impl ProblemResult {
    /// Create a new problem result
    pub fn new(problem_name: impl Into<String>) -> Self {
        Self {
            problem_name: problem_name.into(),
            squad: Vec::new(),
            lineup_ids: Vec::new(),
            bench_ids: Vec::new(),
            captain_id: None,
            vice_captain_id: None,
            objective_value: 0.0,
            total_cost: Cost::ZERO,
            gameweek: None,
        }
    }

    /// Add a player to the result
    pub fn add_player(&mut self, player: SelectedPlayer) {
        if player.is_captain {
            self.captain_id = Some(player.id);
        }
        if player.is_in_lineup {
            self.lineup_ids.push(player.id);
        } else {
            self.bench_ids.push(player.id);
        }
        self.total_cost = self.total_cost + player.cost;
        self.squad.push(player);
    }

    /// Get lineup players
    pub fn lineup(&self) -> impl Iterator<Item = &SelectedPlayer> {
        self.squad.iter().filter(|p| p.is_in_lineup)
    }

    /// Get bench players
    pub fn bench(&self) -> impl Iterator<Item = &SelectedPlayer> {
        self.squad.iter().filter(|p| !p.is_in_lineup)
    }

    /// Get captain
    pub fn captain(&self) -> Option<&SelectedPlayer> {
        self.squad.iter().find(|p| p.is_captain)
    }

    /// Calculate total expected points for lineup
    pub fn lineup_expected_points(&self) -> f64 {
        self.squad.iter()
            .filter(|p| p.is_in_lineup)
            .map(|p| p.gw_points())
            .sum()
    }

    /// Get number of players in squad
    pub fn squad_size(&self) -> usize {
        self.squad.len()
    }

    /// Get number of players in lineup
    pub fn lineup_size(&self) -> usize {
        self.lineup_ids.len()
    }

    /// Sort squad by position and lineup status
    pub fn sort_squad(&mut self) {
        self.squad.sort_by(|a, b| {
            // First by lineup status (lineup first)
            let lineup_cmp = b.is_in_lineup.cmp(&a.is_in_lineup);
            if lineup_cmp != std::cmp::Ordering::Equal {
                return lineup_cmp;
            }
            // Then by position
            let pos_cmp = (a.position as u8).cmp(&(b.position as u8));
            if pos_cmp != std::cmp::Ordering::Equal {
                return pos_cmp;
            }
            // Then by expected points (descending)
            b.expected_points.partial_cmp(&a.expected_points).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

/// Result for iterative squad generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterativeResult {
    /// Iteration number
    pub id: usize,

    /// Squad player IDs
    pub squad: Vec<PlayerId>,

    /// Player web names
    pub players: Vec<String>,

    /// Lineup per gameweek
    pub lineup: std::collections::HashMap<u8, Vec<PlayerId>>,

    /// Bench per gameweek (position -> player_id)
    pub bench: std::collections::HashMap<u8, std::collections::HashMap<u8, PlayerId>>,

    /// Captain per gameweek
    pub captain: std::collections::HashMap<u8, PlayerId>,

    /// Expected points per gameweek per player
    #[serde(rename = "xP")]
    pub expected_points: std::collections::HashMap<u8, Vec<f64>>,

    /// Total cost of squad
    pub total_cost: f64,

    /// Objective values
    pub obj: std::collections::HashMap<String, f64>,

    /// Optimization parameters used
    pub params: std::collections::HashMap<String, f64>,

    /// Element types (positions) of squad players
    pub element_type: Vec<u8>,

    /// Team codes of squad players
    pub team_code: Vec<u8>,

    /// Gameweeks included
    pub weeks: Vec<u8>,
}

impl IterativeResult {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            squad: Vec::new(),
            players: Vec::new(),
            lineup: std::collections::HashMap::new(),
            bench: std::collections::HashMap::new(),
            captain: std::collections::HashMap::new(),
            expected_points: std::collections::HashMap::new(),
            total_cost: 0.0,
            obj: std::collections::HashMap::new(),
            params: std::collections::HashMap::new(),
            element_type: Vec::new(),
            team_code: Vec::new(),
            weeks: Vec::new(),
        }
    }
}
