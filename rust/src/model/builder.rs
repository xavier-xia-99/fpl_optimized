//! Model builder for constructing optimization models

use crate::data::ProblemData;
use crate::model::ObjectiveType;
use crate::types::{Gameweek, PlayerId, Position};
use good_lp::*;
use std::collections::HashMap;

/// Builder for constructing MILP models using good_lp
pub struct ModelBuilder {
    /// Lineup variables: x[player_id]
    lineup_vars: HashMap<PlayerId, Variable>,
    /// Captain variables: y[player_id]
    captain_vars: HashMap<PlayerId, Variable>,
    /// Squad variables: z[player_id]
    squad_vars: HashMap<PlayerId, Variable>,
    /// Multi-gameweek lineup variables: lineup[player_id, gameweek]
    lineup_gw_vars: HashMap<(PlayerId, Gameweek), Variable>,
    /// Multi-gameweek captain variables: captain[player_id, gameweek]
    captain_gw_vars: HashMap<(PlayerId, Gameweek), Variable>,
    /// Bench position variables: bench[player_id, gameweek, position]
    bench_vars: HashMap<(PlayerId, Gameweek, u8), Variable>,
    /// Problem definition for adding constraints
    problem: ProblemVariables,
    /// Constraints to add
    constraints: Vec<Constraint>,
    /// Constraint names for debugging
    constraint_names: Vec<String>,
}

impl ModelBuilder {
    /// Create a new model builder
    pub fn new() -> Self {
        Self {
            lineup_vars: HashMap::new(),
            captain_vars: HashMap::new(),
            squad_vars: HashMap::new(),
            lineup_gw_vars: HashMap::new(),
            captain_gw_vars: HashMap::new(),
            bench_vars: HashMap::new(),
            problem: ProblemVariables::new(),
            constraints: Vec::new(),
            constraint_names: Vec::new(),
        }
    }

    /// Initialize single-gameweek variables for all players
    pub fn init_single_gw_variables(&mut self, data: &ProblemData) {
        for player in &data.players {
            let lineup_var = self.problem.add(variable().binary());
            let captain_var = self.problem.add(variable().binary());
            let squad_var = self.problem.add(variable().binary());

            self.lineup_vars.insert(player.id, lineup_var);
            self.captain_vars.insert(player.id, captain_var);
            self.squad_vars.insert(player.id, squad_var);
        }
    }

    /// Initialize multi-gameweek variables
    pub fn init_multi_gw_variables(&mut self, data: &ProblemData, gameweeks: &[Gameweek]) {
        // Squad variables (shared across gameweeks)
        for player in &data.players {
            let squad_var = self.problem.add(variable().binary());
            self.squad_vars.insert(player.id, squad_var);
        }

        // Per-gameweek variables
        for player in &data.players {
            for &gw in gameweeks {
                let lineup_var = self.problem.add(variable().binary());
                let captain_var = self.problem.add(variable().binary());

                self.lineup_gw_vars.insert((player.id, gw), lineup_var);
                self.captain_gw_vars.insert((player.id, gw), captain_var);

                // Bench position variables (4 bench positions)
                for pos in 1..=4 {
                    let bench_var = self.problem.add(variable().binary());
                    self.bench_vars.insert((player.id, gw, pos), bench_var);
                }
            }
        }
    }

    /// Get lineup variable for a player
    pub fn lineup_var(&self, id: PlayerId) -> Variable {
        *self.lineup_vars.get(&id).expect("Player not initialized")
    }

    /// Get captain variable for a player
    pub fn captain_var(&self, id: PlayerId) -> Variable {
        *self.captain_vars.get(&id).expect("Player not initialized")
    }

    /// Get squad variable for a player
    pub fn squad_var(&self, id: PlayerId) -> Variable {
        *self.squad_vars.get(&id).expect("Player not initialized")
    }

    /// Get lineup variable for a player in a specific gameweek
    pub fn lineup_gw_var(&self, id: PlayerId, gw: Gameweek) -> Variable {
        *self.lineup_gw_vars.get(&(id, gw)).expect("Variable not initialized")
    }

    /// Get captain variable for a player in a specific gameweek
    pub fn captain_gw_var(&self, id: PlayerId, gw: Gameweek) -> Variable {
        *self.captain_gw_vars.get(&(id, gw)).expect("Variable not initialized")
    }

    /// Get bench variable for a player in a specific gameweek and bench position
    pub fn bench_var(&self, id: PlayerId, gw: Gameweek, pos: u8) -> Variable {
        *self.bench_vars.get(&(id, gw, pos)).expect("Variable not initialized")
    }

    /// Check if multi-gameweek variables are initialized
    pub fn has_multi_gw_vars(&self) -> bool {
        !self.lineup_gw_vars.is_empty()
    }

    // ==================== CONSTRAINT METHODS ====================

    /// Add squad size constraint: Σ z[i] = size
    pub fn add_squad_size_constraint(&mut self, data: &ProblemData, size: u8, name: &str) {
        let expr: Expression = data.player_ids()
            .map(|id| self.squad_var(id))
            .sum();
        
        self.constraints.push(constraint!(expr == size as i32));
        self.constraint_names.push(name.to_string());
    }

    /// Add lineup size constraint: Σ x[i] = size
    pub fn add_lineup_size_constraint(&mut self, data: &ProblemData, size: u8, name: &str) {
        let expr: Expression = data.player_ids()
            .map(|id| self.lineup_var(id))
            .sum();
        
        self.constraints.push(constraint!(expr == size as i32));
        self.constraint_names.push(name.to_string());
    }

    /// Add budget constraint: Σ z[i] × cost[i] ≤ budget
    pub fn add_budget_constraint(&mut self, data: &ProblemData, budget: u16, name: &str) {
        let expr: Expression = data.players.iter()
            .map(|p| self.squad_var(p.id) * (p.cost.raw() as i32))
            .sum();
        
        self.constraints.push(constraint!(expr <= budget as i32));
        self.constraint_names.push(name.to_string());
    }

    /// Add team limit constraint: Σ z[i] ≤ limit for each team
    pub fn add_team_limit_constraint(&mut self, data: &ProblemData, limit: u8) {
        for team_code in &data.team_codes {
            let expr: Expression = data.players_in_team(*team_code)
                .iter()
                .map(|&id| self.squad_var(id))
                .sum();
            
            self.constraints.push(constraint!(expr <= limit as i32));
            self.constraint_names.push(format!("team_limit_{}", team_code.raw()));
        }
    }

    /// Add squad exact position constraint: Σ z[i] = squad_select for each position
    pub fn add_squad_position_constraint(&mut self, data: &ProblemData) {
        for pos in Position::all() {
            let expr: Expression = data.players_with_position(pos)
                .iter()
                .map(|&id| self.squad_var(id))
                .sum();
            
            let target = pos.squad_select() as i32;
            self.constraints.push(constraint!(expr == target));
            self.constraint_names.push(format!("squad_exact_{}", pos.short_name()));
        }
    }

    /// Add lineup position min/max constraints
    pub fn add_lineup_position_constraints(&mut self, data: &ProblemData) {
        for pos in Position::all() {
            let players_in_pos = data.players_with_position(pos);
            
            // Min constraint
            let expr_min: Expression = players_in_pos.iter()
                .map(|&id| self.lineup_var(id))
                .sum();
            let min_val = pos.min_play() as i32;
            self.constraints.push(constraint!(expr_min >= min_val));
            self.constraint_names.push(format!("lineup_min_{}", pos.short_name()));
            
            // Max constraint
            let expr_max: Expression = players_in_pos.iter()
                .map(|&id| self.lineup_var(id))
                .sum();
            let max_val = pos.max_play() as i32;
            self.constraints.push(constraint!(expr_max <= max_val));
            self.constraint_names.push(format!("lineup_max_{}", pos.short_name()));
        }
    }

    /// Add single captain constraint: Σ y[i] = 1
    pub fn add_single_captain_constraint(&mut self, data: &ProblemData, name: &str) {
        let expr: Expression = data.player_ids()
            .map(|id| self.captain_var(id))
            .sum();
        
        self.constraints.push(constraint!(expr == 1));
        self.constraint_names.push(name.to_string());
    }

    /// Add captain must play constraint: y[i] ≤ x[i] for all players
    pub fn add_captain_plays_constraints(&mut self, data: &ProblemData) {
        for player in &data.players {
            let captain = self.captain_var(player.id);
            let lineup = self.lineup_var(player.id);
            
            self.constraints.push(constraint!(captain <= lineup));
            self.constraint_names.push(format!("captain_plays_{}", player.id.raw()));
        }
    }

    /// Add lineup subset of squad constraint: x[i] ≤ z[i] for all players
    pub fn add_lineup_squad_constraints(&mut self, data: &ProblemData) {
        for player in &data.players {
            let lineup = self.lineup_var(player.id);
            let squad = self.squad_var(player.id);
            
            self.constraints.push(constraint!(lineup <= squad));
            self.constraint_names.push(format!("lineup_squad_{}", player.id.raw()));
        }
    }

    /// Add differential constraint: z[i] = 0 for high ownership players
    pub fn add_differential_constraint(&mut self, data: &ProblemData, max_ownership: f64) {
        for player in &data.players {
            if player.ownership_percent > max_ownership {
                let squad = self.squad_var(player.id);
                self.constraints.push(constraint!(squad == 0));
                self.constraint_names.push(format!("diff_ban_{}", player.id.raw()));
            }
        }
    }

    /// Add cutoff constraint for iterative solving
    pub fn add_cutoff_constraint(&mut self, prev_squad: &[PlayerId], cutoff: usize, iteration: usize) {
        let expr: Expression = prev_squad.iter()
            .map(|&id| self.squad_var(id))
            .sum();
        let cutoff_val = cutoff as i32;
        
        self.constraints.push(constraint!(expr <= cutoff_val));
        self.constraint_names.push(format!("cutoff_{}", iteration));
    }

    // ==================== BUILD METHODS ====================

    /// Build the objective expression for single gameweek
    pub fn build_single_gw_objective(
        &self,
        data: &ProblemData,
        objective_type: ObjectiveType,
    ) -> Expression {
        match objective_type {
            ObjectiveType::MaximizeExpectedPoints => {
                // Σ points[i] × (x[i] + y[i])
                data.players.iter()
                    .map(|p| {
                        let xp = data.get_expected_points(p.id);
                        let lineup = self.lineup_var(p.id);
                        let captain = self.captain_var(p.id);
                        (lineup + captain) * xp
                    })
                    .sum()
            }
            ObjectiveType::MaximizeWithBenchWeight { bench_weight } => {
                // Σ points[i] × (x[i] + y[i] + w×(z[i] - x[i]))
                data.players.iter()
                    .map(|p| {
                        let xp = data.get_expected_points(p.id);
                        let lineup = self.lineup_var(p.id);
                        let captain = self.captain_var(p.id);
                        let squad = self.squad_var(p.id);
                        lineup * xp + captain * xp + (squad - lineup) * (xp * bench_weight)
                    })
                    .sum()
            }
            ObjectiveType::MaximizeBenchBoost => {
                // Σ points[i] × (z[i] + y[i])
                data.players.iter()
                    .map(|p| {
                        let xp = data.get_expected_points(p.id);
                        let squad = self.squad_var(p.id);
                        let captain = self.captain_var(p.id);
                        (squad + captain) * xp
                    })
                    .sum()
            }
        }
    }

    /// Build the total expected points objective for set-and-forget
    pub fn build_total_gw_objective(
        &self,
        data: &ProblemData,
        objective_type: ObjectiveType,
    ) -> Expression {
        match objective_type {
            ObjectiveType::MaximizeWithBenchWeight { bench_weight } => {
                // Using total expected points across all gameweeks
                data.players.iter()
                    .map(|p| {
                        let total_xp = data.get_total_expected_points(p.id);
                        let lineup = self.lineup_var(p.id);
                        let captain = self.captain_var(p.id);
                        let squad = self.squad_var(p.id);
                        lineup * total_xp + captain * total_xp + (squad - lineup) * (total_xp * bench_weight)
                    })
                    .sum()
            }
            _ => {
                data.players.iter()
                    .map(|p| {
                        let total_xp = data.get_total_expected_points(p.id);
                        let lineup = self.lineup_var(p.id);
                        let captain = self.captain_var(p.id);
                        (lineup + captain) * total_xp
                    })
                    .sum()
            }
        }
    }

    /// Solve the model and return variable values
    pub fn solve(self, objective: Expression) -> Result<SolverSolution, String> {
        let mut model = self.problem
            .maximise(objective)
            .using(default_solver);
        
        // Add constraints one by one
        for constraint in self.constraints {
            model = model.with(constraint);
        }
        
        let result = model.solve();

        match result {
            Ok(solution) => {
                // Extract variable values into our own structure
                let mut lineup_values = HashMap::new();
                let mut captain_values = HashMap::new();
                let mut squad_values = HashMap::new();
                
                for (&id, &var) in &self.lineup_vars {
                    lineup_values.insert(id, solution.value(var) > 0.5);
                }
                for (&id, &var) in &self.captain_vars {
                    captain_values.insert(id, solution.value(var) > 0.5);
                }
                for (&id, &var) in &self.squad_vars {
                    squad_values.insert(id, solution.value(var) > 0.5);
                }
                
                Ok(SolverSolution {
                    lineup_values,
                    captain_values,
                    squad_values,
                })
            }
            Err(e) => Err(format!("Solver error: {:?}", e)),
        }
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Solution from the solver with variable mappings
pub struct SolverSolution {
    lineup_values: HashMap<PlayerId, bool>,
    captain_values: HashMap<PlayerId, bool>,
    squad_values: HashMap<PlayerId, bool>,
}

impl SolverSolution {
    /// Check if player is in lineup
    pub fn is_in_lineup(&self, id: PlayerId) -> bool {
        *self.lineup_values.get(&id).unwrap_or(&false)
    }

    /// Check if player is captain
    pub fn is_captain(&self, id: PlayerId) -> bool {
        *self.captain_values.get(&id).unwrap_or(&false)
    }

    /// Check if player is in squad
    pub fn is_in_squad(&self, id: PlayerId) -> bool {
        *self.squad_values.get(&id).unwrap_or(&false)
    }
}
