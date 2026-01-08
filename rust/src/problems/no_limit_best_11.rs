//! No Limit Best 11 problem - best lineup without budget or team constraints

use crate::constraints::{
    CaptainPlaysConstraint, Constraint, LineupPositionConstraint, LineupSizeConstraint,
    SingleCaptainConstraint,
};
use crate::data::ProblemData;
use crate::error::Result;
use crate::model::{ModelBuilder, ObjectiveType};
use crate::problems::Problem;
use crate::solver::{ProblemResult, SelectedPlayer};

/// No Limit Best 11 problem
///
/// Find the best possible starting 11 without budget or team restrictions.
/// Useful for understanding the theoretical maximum points.
#[derive(Debug, Clone, Default)]
pub struct NoLimitBest11;

impl Problem for NoLimitBest11 {
    fn name(&self) -> &str {
        "No Limit Best 11"
    }

    fn output_name(&self) -> &str {
        "no_limit_best_11"
    }

    fn solve(&self, data: &ProblemData) -> Result<ProblemResult> {
        let mut builder = ModelBuilder::new();
        
        // Initialize single-gameweek variables
        builder.init_single_gw_variables(data);
        
        // Apply constraints
        let constraints: Vec<Box<dyn Constraint>> = vec![
            Box::new(LineupSizeConstraint::default()),
            Box::new(LineupPositionConstraint),
            Box::new(SingleCaptainConstraint),
            Box::new(CaptainPlaysConstraint),
        ];
        
        for constraint in &constraints {
            constraint.apply(&mut builder, data);
        }
        
        // Build objective: maximize expected points
        let objective = builder.build_single_gw_objective(data, ObjectiveType::MaximizeExpectedPoints);
        
        // Solve
        let solution = builder.solve(objective)
            .map_err(|e| crate::error::FplError::Solver(crate::error::SolverError::Internal(e)))?;
        
        // Parse solution
        let mut result = ProblemResult::new(self.name());
        result.gameweek = data.next_gameweek();
        
        let mut total_points = 0.0;
        
        for player in &data.players {
            let is_in_lineup = solution.is_in_lineup(player.id);
            let is_captain = solution.is_captain(player.id);
            
            if is_in_lineup {
                let xp = data.get_expected_points(player.id);
                let multiplier = if is_captain { 2 } else { 1 };
                
                total_points += xp * multiplier as f64;
                
                result.add_player(SelectedPlayer {
                    id: player.id,
                    web_name: player.web_name.clone(),
                    team_code: player.team_code.raw(),
                    position: player.position,
                    cost: player.cost,
                    expected_points: xp,
                    ownership_percent: player.ownership_percent,
                    is_captain,
                    is_in_lineup: true,
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
    use crate::types::{Gameweek, Player, PlayerId, PlayerGameweekProjection, Position, ProjectionData};

    fn create_test_data() -> ProblemData {
        // Create enough players for a valid squad
        let mut players = Vec::new();
        let mut projections = ProjectionData::new();
        
        // 2 GKs
        for i in 1..=2 {
            players.push(Player::new(i, format!("GK{}", i), 1, Position::Goalkeeper, 50));
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), Gameweek(1), 3.0 + i as f64));
        }
        
        // 5 DEFs
        for i in 3..=7 {
            players.push(Player::new(i, format!("DEF{}", i), (i % 5 + 1) as u8, Position::Defender, 55));
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), Gameweek(1), 4.0 + i as f64));
        }
        
        // 5 MIDs
        for i in 8..=12 {
            players.push(Player::new(i, format!("MID{}", i), (i % 5 + 1) as u8, Position::Midfielder, 70));
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), Gameweek(1), 6.0 + i as f64));
        }
        
        // 3 FWDs
        for i in 13..=15 {
            players.push(Player::new(i, format!("FWD{}", i), (i % 3 + 1) as u8, Position::Forward, 80));
            projections.insert(PlayerGameweekProjection::new(PlayerId(i), Gameweek(1), 7.0 + i as f64));
        }
        
        ProblemData::new(players, projections)
    }

    #[test]
    fn test_no_limit_best_11_lineup_size() {
        let data = create_test_data();
        let problem = NoLimitBest11;
        
        let result = problem.solve(&data).unwrap();
        
        assert_eq!(result.lineup_size(), 11);
    }

    #[test]
    fn test_no_limit_best_11_has_captain() {
        let data = create_test_data();
        let problem = NoLimitBest11;
        
        let result = problem.solve(&data).unwrap();
        
        assert!(result.captain_id.is_some());
    }
}
