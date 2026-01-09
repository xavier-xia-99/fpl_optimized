//! Limited Best Squad problem - full squad with budget and team limits

use crate::constraints::limited_squad_constraints;
use crate::data::ProblemData;
use crate::error::Result;
use crate::model::{ModelBuilder, ObjectiveType};
use crate::problems::Problem;
use crate::solver::{ProblemResult, SelectedPlayer};
use crate::types::Cost;

/// Limited Best Squad problem
///
/// Find the best possible 15-player squad with:
/// - Budget constraint (default £100M)
/// - Team limit (max 3 per team)
/// - Position requirements
#[derive(Debug, Clone)]
pub struct LimitedBestSquad {
    pub budget: Cost,
}

impl Default for LimitedBestSquad {
    fn default() -> Self {
        Self {
            budget: Cost::BUDGET_DEFAULT,
        }
    }
}

impl LimitedBestSquad {
    /// Create with custom budget
    pub fn with_budget(budget: Cost) -> Self {
        Self { budget }
    }
}

impl Problem for LimitedBestSquad {
    fn name(&self) -> &str {
        "Limited Best Squad"
    }

    fn output_name(&self) -> &str {
        "limited_best_15"
    }

    fn solve(&self, data: &ProblemData) -> Result<ProblemResult> {
        let mut builder = ModelBuilder::new();
        
        // Initialize variables
        builder.init_single_gw_variables(data);
        
        // Apply all limited squad constraints
        let constraints = limited_squad_constraints(self.budget.raw());
        for constraint in &constraints {
            constraint.apply(&mut builder, data);
        }
        
        // Build objective
        let objective = builder.build_single_gw_objective(data, ObjectiveType::MaximizeExpectedPoints);
        
        // Solve
        let solution = builder.solve(objective)
            .map_err(|e| crate::error::FplError::Solver(crate::error::SolverError::Internal(e)))?;
        
        // Parse solution
        let mut result = ProblemResult::new(self.name());
        result.gameweek = data.next_gameweek();
        
        let mut total_points = 0.0;
        
        for player in &data.players {
            let is_in_squad = solution.is_in_squad(player.id);
            let is_in_lineup = solution.is_in_lineup(player.id);
            let is_captain = solution.is_captain(player.id);
            
            if is_in_squad {
                let xp = data.get_expected_points(player.id);
                let multiplier = if is_captain { 2 } else if is_in_lineup { 1 } else { 0 };
                
                if is_in_lineup {
                    total_points += xp * multiplier as f64;
                }
                
                result.add_player(SelectedPlayer {
                    id: player.id,
                    web_name: player.web_name.clone(),
                    team_code: player.team_code.raw(),
                    position: player.position,
                    cost: player.cost,
                    expected_points: xp,
                    ownership_percent: player.ownership_percent,
                    is_captain,
                    is_in_lineup,
                    multiplier,
                });
            }
        }
        
        result.objective_value = total_points;
        result.sort_squad();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Gameweek, Player, PlayerId, PlayerGameweekProjection, Position, ProjectionData, TeamCode};

    fn create_test_data() -> ProblemData {
        let mut players = Vec::new();
        let mut projections = ProjectionData::new();
        let gw = Gameweek(1);
        
        // Create enough players with varying teams
        // 4 GKs
        for i in 1..=4 {
            let mut p = Player::new(i, format!("GK{}", i), (i % 4 + 1) as u8, Position::Goalkeeper, 45);
            players.push(p);
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), gw, 3.0 + (i as f64) * 0.1));
        }
        
        // 10 DEFs
        for i in 5..=14 {
            let mut p = Player::new(i, format!("DEF{}", i), ((i - 5) % 10 + 1) as u8, Position::Defender, 50);
            players.push(p);
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), gw, 4.0 + (i as f64) * 0.1));
        }
        
        // 10 MIDs
        for i in 15..=24 {
            let mut p = Player::new(i, format!("MID{}", i), ((i - 15) % 10 + 1) as u8, Position::Midfielder, 70);
            players.push(p);
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), gw, 6.0 + (i as f64) * 0.1));
        }
        
        // 6 FWDs
        for i in 25..=30 {
            let mut p = Player::new(i, format!("FWD{}", i), ((i - 25) % 6 + 1) as u8, Position::Forward, 80);
            players.push(p);
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), gw, 7.0 + (i as f64) * 0.1));
        }
        
        ProblemData::new(players, projections)
    }

    #[test]
    fn test_limited_squad_size() {
        let data = create_test_data();
        let problem = LimitedBestSquad::default();
        
        let result = problem.solve(&data).unwrap();
        
        assert_eq!(result.squad_size(), 15);
    }

    #[test]
    fn test_limited_lineup_size() {
        let data = create_test_data();
        let problem = LimitedBestSquad::default();
        
        let result = problem.solve(&data).unwrap();
        
        assert_eq!(result.lineup_size(), 11);
    }

    #[test]
    fn test_limited_budget_respected() {
        let data = create_test_data();
        let problem = LimitedBestSquad::default();
        
        let result = problem.solve(&data).unwrap();
        
        assert!(result.total_cost <= Cost::BUDGET_DEFAULT);
    }
}
