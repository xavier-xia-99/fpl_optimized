//! Problem variants for FPL optimization

mod limited_best_squad;
mod no_limit_best_11;
mod weighted_bench;
mod bench_boost;
mod differential;
mod set_and_forget;
mod iterative;

pub use bench_boost::BenchBoostSquad;
pub use differential::DifferentialSquad;
pub use iterative::{IterativeConfig, IterativeSquads};
pub use limited_best_squad::LimitedBestSquad;
pub use no_limit_best_11::NoLimitBest11;
pub use set_and_forget::SetAndForget;
pub use weighted_bench::WeightedBenchSquad;

use crate::data::ProblemData;
use crate::error::Result;
use crate::model::ObjectiveType;
use crate::solver::ProblemResult;

/// Trait for FPL optimization problems
pub trait Problem {
    /// Problem name
    fn name(&self) -> &str;

    /// Output file name (without extension)
    fn output_name(&self) -> &str;

    /// Solve the problem and return results
    fn solve(&self, data: &ProblemData) -> Result<ProblemResult>;

    /// Get the objective type
    fn objective_type(&self) -> ObjectiveType {
        ObjectiveType::MaximizeExpectedPoints
    }
}
