//! Iterative Squads problem - generate multiple diverse squads

use crate::constraints::limited_squad_constraints;
use crate::data::ProblemData;
use crate::error::{FplError, Result, SolverError};
use crate::model::{ModelBuilder, ObjectiveType};
use crate::solver::{IterativeResult, ProblemResult, SelectedPlayer};
use crate::types::{Cost, PlayerId};
use rand::prelude::*;

/// Configuration for iterative squad generation
#[derive(Debug, Clone)]
pub struct IterativeConfig {
    /// Number of squads to generate
    pub iterations: usize,
    /// Maximum overlap with previous solution
    pub cutoff: usize,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Budget
    pub budget: Cost,
}

impl Default for IterativeConfig {
    fn default() -> Self {
        Self {
            iterations: 50,
            cutoff: 12,
            seed: Some(42),
            budget: Cost::BUDGET_DEFAULT,
        }
    }
}

/// Iterative Squads problem
///
/// Generate multiple diverse squad solutions using cutoff constraints
/// to force variety between solutions.
#[derive(Debug, Clone)]
pub struct IterativeSquads {
    pub config: IterativeConfig,
}

impl Default for IterativeSquads {
    fn default() -> Self {
        Self {
            config: IterativeConfig::default(),
        }
    }
}

impl IterativeSquads {
    /// Create with custom configuration
    pub fn new(config: IterativeConfig) -> Self {
        Self { config }
    }

    /// Create with specified iteration count
    pub fn with_iterations(iterations: usize) -> Self {
        Self {
            config: IterativeConfig {
                iterations,
                ..Default::default()
            },
        }
    }

    /// Solve iteratively and return all results
    pub fn solve_all(&self, data: &ProblemData) -> Result<Vec<IterativeResult>> {
        let mut rng = match self.config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let mut all_results = Vec::with_capacity(self.config.iterations);
        let mut prev_squad: Vec<PlayerId> = Vec::new();

        for iteration in 0..self.config.iterations {
            let result = self.solve_single_iteration(data, iteration, &prev_squad, &mut rng)?;
            
            // Store squad for next iteration's cutoff constraint
            prev_squad = result.squad.clone();
            
            all_results.push(result);
        }

        Ok(all_results)
    }

    /// Solve a single iteration
    fn solve_single_iteration(
        &self,
        data: &ProblemData,
        iteration: usize,
        prev_squad: &[PlayerId],
        rng: &mut StdRng,
    ) -> Result<IterativeResult> {
        let mut builder = ModelBuilder::new();
        
        // Initialize variables
        builder.init_single_gw_variables(data);
        
        // Apply standard constraints
        let constraints = limited_squad_constraints(self.config.budget.raw());
        for constraint in &constraints {
            constraint.apply(&mut builder, data);
        }
        
        // Add cutoff constraint if not first iteration
        if iteration > 0 && !prev_squad.is_empty() {
            builder.add_cutoff_constraint(prev_squad, self.config.cutoff, iteration);
        }
        
        // Generate randomized weights for objective diversity
        let weights = IterativeWeights::random(rng);
        
        // Build objective (use standard for now, could add weighted combination)
        let objective = builder.build_single_gw_objective(
            data,
            ObjectiveType::MaximizeWithBenchWeight { bench_weight: 0.1 },
        );
        
        // Solve
        let solution = builder.solve(objective)
            .map_err(|e| FplError::Solver(SolverError::Internal(e)))?;
        
        // Parse solution into IterativeResult
        let mut result = IterativeResult::new(iteration);
        result.params = weights.to_params();
        
        let mut total_cost = 0.0;
        
        for player in &data.players {
            if solution.is_in_squad(player.id) {
                result.squad.push(player.id);
                result.players.push(player.web_name.clone());
                result.element_type.push(player.position as u8);
                result.team_code.push(player.team_code.raw());
                total_cost += player.cost.to_millions() * 10.0;
            }
        }
        
        result.total_cost = total_cost;
        
        // Store gameweeks
        for gw in data.gameweeks() {
            result.weeks.push(gw.raw());
        }
        
        // Store lineups, captains, expected points per gameweek
        if let Some(gw) = data.next_gameweek() {
            let gw_num = gw.raw();
            let mut lineup = Vec::new();
            let mut xp_values = Vec::new();
            
            for &player_id in &result.squad {
                let xp = data.get_expected_points(player_id);
                xp_values.push(xp);
                
                if solution.is_in_lineup(player_id) {
                    lineup.push(player_id);
                }
                
                if solution.is_captain(player_id) {
                    result.captain.insert(gw_num, player_id);
                }
            }
            
            result.lineup.insert(gw_num, lineup);
            result.expected_points.insert(gw_num, xp_values);
        }
        
        // Calculate objective value
        let obj_value: f64 = result.squad.iter()
            .zip(result.expected_points.values().next().unwrap_or(&vec![]))
            .map(|(_, &xp)| xp)
            .sum();
        result.obj.insert("overall".to_string(), obj_value);
        
        Ok(result)
    }

    /// Solve and return as standard ProblemResults
    pub fn solve_as_problem_results(&self, data: &ProblemData) -> Result<Vec<ProblemResult>> {
        let iterative_results = self.solve_all(data)?;
        
        Ok(iterative_results.into_iter().enumerate().map(|(i, ir)| {
            let mut result = ProblemResult::new(format!("Iterative Squad {}", i));
            result.gameweek = data.next_gameweek();
            
            for (idx, &player_id) in ir.squad.iter().enumerate() {
                if let Some(player) = data.get_player(player_id) {
                    let xp = ir.expected_points
                        .values()
                        .next()
                        .and_then(|v| v.get(idx))
                        .copied()
                        .unwrap_or(0.0);
                    
                    let is_captain = ir.captain.values().any(|&id| id == player_id);
                    let is_in_lineup = ir.lineup.values().any(|lineup| lineup.contains(&player_id));
                    
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
                        multiplier: if is_captain { 2 } else if is_in_lineup { 1 } else { 0 },
                    });
                }
            }
            
            result.objective_value = *ir.obj.get("overall").unwrap_or(&0.0);
            result.sort_squad();
            result
        }).collect())
    }
}

/// Random weights for objective function diversity
#[derive(Debug, Clone)]
struct IterativeWeights {
    pts_weight: f64,
    cost_weight: f64,
    xmin_weight: f64,
    ep_weight: f64,
    form_weight: f64,
    ppg_weight: f64,
    bps_weight: f64,
    ict_weight: f64,
    rawxp_weight: f64,
    ownership_weight: f64,
}

impl IterativeWeights {
    fn random(rng: &mut StdRng) -> Self {
        Self {
            pts_weight: rng.gen(),
            cost_weight: rng.gen(),
            xmin_weight: rng.gen(),
            ep_weight: rng.gen(),
            form_weight: rng.gen(),
            ppg_weight: rng.gen(),
            bps_weight: rng.gen(),
            ict_weight: rng.gen(),
            rawxp_weight: rng.gen(),
            ownership_weight: rng.gen_range(-1.0..1.0),
        }
    }

    fn to_params(&self) -> std::collections::HashMap<String, f64> {
        let mut params = std::collections::HashMap::new();
        params.insert("pts_weight".to_string(), self.pts_weight);
        params.insert("cost_weight".to_string(), self.cost_weight);
        params.insert("xmin_weight".to_string(), self.xmin_weight);
        params.insert("ep_weight".to_string(), self.ep_weight);
        params.insert("form_weight".to_string(), self.form_weight);
        params.insert("ppg_weight".to_string(), self.ppg_weight);
        params.insert("bps_weight".to_string(), self.bps_weight);
        params.insert("ict_weight".to_string(), self.ict_weight);
        params.insert("rawxp_weight".to_string(), self.rawxp_weight);
        params.insert("ownership_weight".to_string(), self.ownership_weight);
        params
    }
}
