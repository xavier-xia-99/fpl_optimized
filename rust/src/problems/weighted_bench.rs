//! Weighted Bench Squad problem - values bench players at a fraction

use crate::constraints::limited_squad_constraints;
use crate::data::ProblemData;
use crate::error::Result;
use crate::model::{ModelBuilder, ObjectiveType};
use crate::problems::Problem;
use crate::solver::{ProblemResult, SelectedPlayer};
use crate::types::Cost;

/// Weighted Bench Squad problem
///
/// Find the best squad where bench players contribute to the objective
/// at a reduced weight (default 0.1).
#[derive(Debug, Clone)]
pub struct WeightedBenchSquad {
    pub budget: Cost,
    pub bench_weight: f64,
}

impl Default for WeightedBenchSquad {
    fn default() -> Self {
        Self {
            budget: Cost::BUDGET_DEFAULT,
            bench_weight: 0.1,
        }
    }
}

impl WeightedBenchSquad {
    /// Create with custom parameters
    pub fn new(budget: Cost, bench_weight: f64) -> Self {
        Self { budget, bench_weight }
    }
}

impl Problem for WeightedBenchSquad {
    fn name(&self) -> &str {
        "Weighted Bench Squad"
    }

    fn output_name(&self) -> &str {
        "limited_best_15_weighted"
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
        
        // Build objective with bench weight
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
                let multiplier = if is_captain { 2 } else if is_in_lineup { 1 } else { 0 };
                
                // Calculate effective points (including bench contribution)
                let effective_points = if is_in_lineup {
                    xp * multiplier as f64
                } else {
                    xp * self.bench_weight
                };
                total_points += effective_points;
                
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
