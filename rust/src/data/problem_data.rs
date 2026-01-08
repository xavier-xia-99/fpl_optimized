//! Problem data structure for optimization

use crate::types::{Gameweek, Player, PlayerId, Position, ProjectionData, TeamCode};
use std::collections::{HashMap, HashSet};

/// All data needed for solving an optimization problem
#[derive(Debug)]
pub struct ProblemData {
    /// All players
    pub players: Vec<Player>,

    /// Player ID to index mapping
    player_index: HashMap<PlayerId, usize>,

    /// Projections for all player-gameweek combinations
    pub projections: ProjectionData,

    /// Unique team codes
    pub team_codes: Vec<TeamCode>,

    /// Players grouped by position
    players_by_position: HashMap<Position, Vec<PlayerId>>,

    /// Players grouped by team
    players_by_team: HashMap<TeamCode, Vec<PlayerId>>,
}

impl ProblemData {
    /// Create problem data from players and projections
    pub fn new(players: Vec<Player>, projections: ProjectionData) -> Self {
        let mut player_index = HashMap::new();
        let mut team_codes_set = HashSet::new();
        let mut players_by_position: HashMap<Position, Vec<PlayerId>> = HashMap::new();
        let mut players_by_team: HashMap<TeamCode, Vec<PlayerId>> = HashMap::new();

        for (idx, player) in players.iter().enumerate() {
            player_index.insert(player.id, idx);
            team_codes_set.insert(player.team_code);

            players_by_position
                .entry(player.position)
                .or_default()
                .push(player.id);

            players_by_team
                .entry(player.team_code)
                .or_default()
                .push(player.id);
        }

        let team_codes: Vec<_> = team_codes_set.into_iter().collect();

        Self {
            players,
            player_index,
            projections,
            team_codes,
            players_by_position,
            players_by_team,
        }
    }

    /// Get a player by ID
    pub fn get_player(&self, id: PlayerId) -> Option<&Player> {
        self.player_index.get(&id).map(|&idx| &self.players[idx])
    }

    /// Get player index by ID
    pub fn get_player_index(&self, id: PlayerId) -> Option<usize> {
        self.player_index.get(&id).copied()
    }

    /// Get all player IDs
    pub fn player_ids(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.players.iter().map(|p| p.id)
    }

    /// Get players by position
    pub fn players_with_position(&self, position: Position) -> &[PlayerId] {
        self.players_by_position
            .get(&position)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get players by team
    pub fn players_in_team(&self, team_code: TeamCode) -> &[PlayerId] {
        self.players_by_team
            .get(&team_code)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get expected points for a player in the first gameweek
    pub fn get_expected_points(&self, player_id: PlayerId) -> f64 {
        self.projections.next_gw_expected_points(player_id)
    }

    /// Get total expected points across all gameweeks
    pub fn get_total_expected_points(&self, player_id: PlayerId) -> f64 {
        self.projections.total_expected_points(player_id)
    }

    /// Get expected points for a specific gameweek
    pub fn get_gameweek_expected_points(&self, player_id: PlayerId, gw: Gameweek) -> f64 {
        self.projections.get_expected_points(player_id, gw)
    }

    /// Get gameweeks in projections
    pub fn gameweeks(&self) -> &[Gameweek] {
        self.projections.gameweeks()
    }

    /// Get next (first) gameweek
    pub fn next_gameweek(&self) -> Option<Gameweek> {
        self.projections.first_gameweek()
    }

    /// Get number of players
    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    /// Get number of teams
    pub fn num_teams(&self) -> usize {
        self.team_codes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Cost, PlayerGameweekProjection};

    fn create_test_player(id: u32, position: Position, team: u8) -> Player {
        Player {
            id: PlayerId(id),
            web_name: format!("Player{}", id),
            full_name: None,
            team_code: TeamCode(team),
            position,
            cost: Cost(100),
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

    #[test]
    fn test_problem_data_creation() {
        let players = vec![
            create_test_player(1, Position::Goalkeeper, 1),
            create_test_player(2, Position::Defender, 2),
            create_test_player(3, Position::Midfielder, 1),
        ];
        let projections = ProjectionData::new();

        let data = ProblemData::new(players, projections);

        assert_eq!(data.num_players(), 3);
        assert_eq!(data.num_teams(), 2);
    }

    #[test]
    fn test_get_player() {
        let players = vec![
            create_test_player(1, Position::Goalkeeper, 1),
            create_test_player(2, Position::Defender, 2),
        ];
        let data = ProblemData::new(players, ProjectionData::new());

        let player = data.get_player(PlayerId(1)).unwrap();
        assert_eq!(player.web_name, "Player1");

        assert!(data.get_player(PlayerId(99)).is_none());
    }

    #[test]
    fn test_players_by_position() {
        let players = vec![
            create_test_player(1, Position::Goalkeeper, 1),
            create_test_player(2, Position::Defender, 1),
            create_test_player(3, Position::Defender, 2),
        ];
        let data = ProblemData::new(players, ProjectionData::new());

        assert_eq!(data.players_with_position(Position::Goalkeeper).len(), 1);
        assert_eq!(data.players_with_position(Position::Defender).len(), 2);
        assert_eq!(data.players_with_position(Position::Forward).len(), 0);
    }

    #[test]
    fn test_players_by_team() {
        let players = vec![
            create_test_player(1, Position::Goalkeeper, 1),
            create_test_player(2, Position::Defender, 1),
            create_test_player(3, Position::Midfielder, 2),
        ];
        let data = ProblemData::new(players, ProjectionData::new());

        assert_eq!(data.players_in_team(TeamCode(1)).len(), 2);
        assert_eq!(data.players_in_team(TeamCode(2)).len(), 1);
    }

    #[test]
    fn test_expected_points() {
        let players = vec![create_test_player(1, Position::Midfielder, 1)];
        let mut projections = ProjectionData::new();
        projections.insert(PlayerGameweekProjection::new(PlayerId(1), Gameweek(21), 5.5));

        let data = ProblemData::new(players, projections);

        assert_eq!(data.get_expected_points(PlayerId(1)), 5.5);
        assert_eq!(data.get_expected_points(PlayerId(99)), 0.0);
    }
}
