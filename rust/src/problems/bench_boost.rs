//! Bench Boost Squad problem - all 15 players count

use crate::constraints::limited_squad_constraints;
use crate::data::ProblemData;
use crate::error::Result;
use crate::model::{ModelBuilder, ObjectiveType};
use crate::problems::Problem;
use crate::solver::{ProblemResult, SelectedPlayer};
use crate::types::Cost;

/// Bench Boost Squad problem
///
/// Find the best squad for when bench boost chip is active.
/// All 15 players contribute fully to points.
#[derive(Debug, Clone)]
pub struct BenchBoostSquad {
    pub budget: Cost,
}

impl Default for BenchBoostSquad {
    fn default() -> Self {
        Self {
            budget: Cost::BUDGET_DEFAULT,
        }
    }
}

impl BenchBoostSquad {
    /// Create with custom budget
    pub fn with_budget(budget: Cost) -> Self {
        Self { budget }
    }
}

impl Problem for BenchBoostSquad {
    fn name(&self) -> &str {
        "Bench Boost Squad"
    }

    fn output_name(&self) -> &str {
        "limited_best_15_bb"
    }

    fn objective_type(&self) -> ObjectiveType {
        ObjectiveType::MaximizeBenchBoost
    }

    fn solve(&self, data: &ProblemData) -> Result<ProblemResult> {
        let mut builder = ModelBuilder::new();
        
        // Initialize variables
        builder.init_single_gw_variables(data);
        
        // Apply constraints
        let constraints = limited_squad_constraints(self.budget.raw());
        for constraint in &constraints {
            constraint.apply(&mut builder, data);
        }
        
        // Build bench boost objective: maximize Σ points × (z + y)
        let objective = builder.build_single_gw_objective(data, self.objective_type());
        
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
                // In bench boost, all players count
                // Captain still gets double
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
