//! Set and Forget problem - optimize across multiple gameweeks

use crate::constraints::limited_squad_constraints;
use crate::data::ProblemData;
use crate::error::Result;
use crate::model::{ModelBuilder, ObjectiveType};
use crate::problems::Problem;
use crate::solver::{ProblemResult, SelectedPlayer};
use crate::types::Cost;

/// Set and Forget problem
///
/// Find the best squad optimized for points across all available gameweeks.
/// Uses total expected points as the objective.
#[derive(Debug, Clone)]
pub struct SetAndForget {
    pub budget: Cost,
    pub bench_weight: f64,
}

impl Default for SetAndForget {
    fn default() -> Self {
        Self {
            budget: Cost::BUDGET_DEFAULT,
            bench_weight: 0.1,
        }
    }
}

impl SetAndForget {
    /// Create with custom parameters
    pub fn new(budget: Cost, bench_weight: f64) -> Self {
        Self { budget, bench_weight }
    }
}

impl Problem for SetAndForget {
    fn name(&self) -> &str {
        "Set and Forget"
    }

    fn output_name(&self) -> &str {
        "limited_best_set_and_forget"
    }

    fn objective_type(&self) -> ObjectiveType {
        ObjectiveType::MaximizeWithBenchWeight { bench_weight: self.bench_weight }
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
        
        // Build objective using TOTAL expected points across all gameweeks
        let objective = builder.build_total_gw_objective(data, self.objective_type());
        
        // Solve
        let solution = builder.solve(objective)
            .map_err(|e| crate::error::FplError::Solver(crate::error::SolverError::Internal(e)))?;
        
        // Parse solution
        let mut result = ProblemResult::new(self.name());
        
        let num_gameweeks = data.gameweeks().len();
        let mut total_points = 0.0;
        
        for player in &data.players {
            let is_in_squad = solution.is_in_squad(player.id);
            let is_in_lineup = solution.is_in_lineup(player.id);
            let is_captain = solution.is_captain(player.id);
            
            if is_in_squad {
                // Use average expected points per gameweek for display
                let total_xp = data.get_total_expected_points(player.id);
                let avg_xp = if num_gameweeks > 0 {
                    total_xp / num_gameweeks as f64
                } else {
                    total_xp
                };
                
                let multiplier = if is_captain { 2 } else if is_in_lineup { 1 } else { 0 };
                
                let effective_points = if is_in_lineup {
                    total_xp * multiplier as f64
                } else {
                    total_xp * self.bench_weight
                };
                total_points += effective_points;
                
                result.add_player(SelectedPlayer {
                    id: player.id,
                    web_name: player.web_name.clone(),
                    team_code: player.team_code.raw(),
                    position: player.position,
                    cost: player.cost,
                    expected_points: avg_xp, // Show average per GW
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
