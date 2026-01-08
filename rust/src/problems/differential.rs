//! Differential Squad problem - only low-ownership players

use crate::constraints::{limited_squad_constraints, Constraint, DifferentialConstraint};
use crate::data::ProblemData;
use crate::error::Result;
use crate::model::{ModelBuilder, ObjectiveType};
use crate::problems::Problem;
use crate::solver::{ProblemResult, SelectedPlayer};
use crate::types::Cost;

/// Differential Squad problem
///
/// Find the best squad using only players with ownership below threshold.
/// Default threshold is 5%.
#[derive(Debug, Clone)]
pub struct DifferentialSquad {
    pub budget: Cost,
    pub max_ownership: f64,
    pub bench_weight: f64,
}

impl Default for DifferentialSquad {
    fn default() -> Self {
        Self {
            budget: Cost::BUDGET_DEFAULT,
            max_ownership: 5.0,
            bench_weight: 0.1,
        }
    }
}

impl DifferentialSquad {
    /// Create with custom parameters
    pub fn new(budget: Cost, max_ownership: f64) -> Self {
        Self {
            budget,
            max_ownership,
            bench_weight: 0.1,
        }
    }
}

impl Problem for DifferentialSquad {
    fn name(&self) -> &str {
        "Differential Squad"
    }

    fn output_name(&self) -> &str {
        "limited_best_differential"
    }

    fn objective_type(&self) -> ObjectiveType {
        ObjectiveType::MaximizeWithBenchWeight { bench_weight: self.bench_weight }
    }

    fn solve(&self, data: &ProblemData) -> Result<ProblemResult> {
        let mut builder = ModelBuilder::new();
        
        // Initialize variables
        builder.init_single_gw_variables(data);
        
        // Apply standard constraints
        let constraints = limited_squad_constraints(self.budget.raw());
        for constraint in &constraints {
            constraint.apply(&mut builder, data);
        }
        
        // Apply differential constraint
        let diff_constraint = DifferentialConstraint { max_ownership: self.max_ownership };
        diff_constraint.apply(&mut builder, data);
        
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
