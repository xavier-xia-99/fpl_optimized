//! Optimization constraints for FPL problems
//!
//! This module provides reusable constraint building blocks for FPL optimization.

use crate::data::ProblemData;
use crate::model::ModelBuilder;

/// Trait for constraints that can be applied to a model
pub trait Constraint {
    /// Unique name for this constraint
    fn name(&self) -> &'static str;

    /// Apply this constraint to the model builder
    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData);

    /// Whether this constraint is required (hard) or optional (soft)
    fn is_required(&self) -> bool {
        true
    }
}

/// Squad size constraint: exactly 15 players
#[derive(Debug, Clone)]
pub struct SquadSizeConstraint {
    pub size: u8,
}

impl Default for SquadSizeConstraint {
    fn default() -> Self {
        Self { size: 15 }
    }
}

impl Constraint for SquadSizeConstraint {
    fn name(&self) -> &'static str {
        "squad_limit"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_squad_size_constraint(data, self.size, self.name());
    }
}

/// Lineup size constraint: exactly 11 players
#[derive(Debug, Clone)]
pub struct LineupSizeConstraint {
    pub size: u8,
}

impl Default for LineupSizeConstraint {
    fn default() -> Self {
        Self { size: 11 }
    }
}

impl Constraint for LineupSizeConstraint {
    fn name(&self) -> &'static str {
        "lineup_limit"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_lineup_size_constraint(data, self.size, self.name());
    }
}

/// Budget constraint: total cost ≤ budget
#[derive(Debug, Clone)]
pub struct BudgetConstraint {
    pub budget: u16,
}

impl Default for BudgetConstraint {
    fn default() -> Self {
        Self { budget: 1000 }
    }
}

impl Constraint for BudgetConstraint {
    fn name(&self) -> &'static str {
        "total_cost"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_budget_constraint(data, self.budget, self.name());
    }
}

/// Team limit constraint: max 3 players per team
#[derive(Debug, Clone)]
pub struct TeamLimitConstraint {
    pub limit: u8,
}

impl Default for TeamLimitConstraint {
    fn default() -> Self {
        Self { limit: 3 }
    }
}

impl Constraint for TeamLimitConstraint {
    fn name(&self) -> &'static str {
        "team_limit"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_team_limit_constraint(data, self.limit);
    }
}

/// Squad position exact constraint: 2 GK, 5 DEF, 5 MID, 3 FWD
#[derive(Debug, Clone, Default)]
pub struct SquadPositionConstraint;

impl Constraint for SquadPositionConstraint {
    fn name(&self) -> &'static str {
        "squad_exact"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_squad_position_constraint(data);
    }
}

/// Lineup position min/max constraints
#[derive(Debug, Clone, Default)]
pub struct LineupPositionConstraint;

impl Constraint for LineupPositionConstraint {
    fn name(&self) -> &'static str {
        "lineup_position"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_lineup_position_constraints(data);
    }
}

/// Single captain constraint: exactly 1 captain
#[derive(Debug, Clone, Default)]
pub struct SingleCaptainConstraint;

impl Constraint for SingleCaptainConstraint {
    fn name(&self) -> &'static str {
        "single_captain"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_single_captain_constraint(data, self.name());
    }
}

/// Captain must play constraint: captain must be in lineup
#[derive(Debug, Clone, Default)]
pub struct CaptainPlaysConstraint;

impl Constraint for CaptainPlaysConstraint {
    fn name(&self) -> &'static str {
        "captain_plays"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_captain_plays_constraints(data);
    }
}

/// Lineup subset of squad constraint: lineup players must be in squad
#[derive(Debug, Clone, Default)]
pub struct LineupSquadConstraint;

impl Constraint for LineupSquadConstraint {
    fn name(&self) -> &'static str {
        "lineup_squad"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_lineup_squad_constraints(data);
    }
}

/// Differential constraint: only allow players under ownership threshold
#[derive(Debug, Clone)]
pub struct DifferentialConstraint {
    pub max_ownership: f64,
}

impl Default for DifferentialConstraint {
    fn default() -> Self {
        Self { max_ownership: 5.0 }
    }
}

impl Constraint for DifferentialConstraint {
    fn name(&self) -> &'static str {
        "differential"
    }

    fn apply(&self, builder: &mut ModelBuilder, data: &ProblemData) {
        builder.add_differential_constraint(data, self.max_ownership);
    }
}

/// Standard FPL constraints for lineup problems without squad
pub fn lineup_only_constraints() -> Vec<Box<dyn Constraint>> {
    vec![
        Box::new(LineupSizeConstraint::default()),
        Box::new(LineupPositionConstraint),
        Box::new(SingleCaptainConstraint),
        Box::new(CaptainPlaysConstraint),
    ]
}

/// Standard FPL constraints for limited squad problems
pub fn limited_squad_constraints(budget: u16) -> Vec<Box<dyn Constraint>> {
    vec![
        Box::new(SquadSizeConstraint::default()),
        Box::new(LineupSizeConstraint::default()),
        Box::new(BudgetConstraint { budget }),
        Box::new(TeamLimitConstraint::default()),
        Box::new(SquadPositionConstraint),
        Box::new(LineupPositionConstraint),
        Box::new(SingleCaptainConstraint),
        Box::new(CaptainPlaysConstraint),
        Box::new(LineupSquadConstraint),
    ]
}
