//! Objective function types for optimization

/// Types of objective functions available
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectiveType {
    /// Maximize expected points: Σ points[i] × (x[i] + y[i])
    MaximizeExpectedPoints,

    /// With bench weight: Σ points[i] × (x[i] + y[i] + w×(z[i] - x[i]))
    MaximizeWithBenchWeight { bench_weight: f64 },

    /// Bench boost: Σ points[i] × (z[i] + y[i])
    MaximizeBenchBoost,
}

impl Default for ObjectiveType {
    fn default() -> Self {
        ObjectiveType::MaximizeExpectedPoints
    }
}

impl ObjectiveType {
    /// Create weighted bench objective with specified weight
    pub fn weighted_bench(weight: f64) -> Self {
        ObjectiveType::MaximizeWithBenchWeight { bench_weight: weight }
    }

    /// Get the bench weight (0.0 for standard objectives)
    pub fn bench_weight(&self) -> f64 {
        match self {
            ObjectiveType::MaximizeWithBenchWeight { bench_weight } => *bench_weight,
            _ => 0.0,
        }
    }
}
