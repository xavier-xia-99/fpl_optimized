//! Player cost type

use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Sub};

/// Player cost in tenths of millions (e.g., 100 = £10.0M)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Cost(pub u16);

impl Cost {
    /// Default FPL budget: £100.0M = 1000 tenths
    pub const BUDGET_DEFAULT: Cost = Cost(1000);

    /// Minimum player cost: £4.0M
    pub const MIN: Cost = Cost(40);

    /// Maximum player cost: ~£15.0M
    pub const MAX: Cost = Cost(150);

    /// Zero cost
    pub const ZERO: Cost = Cost(0);

    /// Create a new cost value
    pub const fn new(value: u16) -> Self {
        Cost(value)
    }

    /// Create from millions (e.g., 10.5 -> Cost(105))
    pub fn from_millions(millions: f64) -> Self {
        Cost((millions * 10.0).round() as u16)
    }

    /// Convert to millions (e.g., Cost(105) -> 10.5)
    pub fn to_millions(&self) -> f64 {
        self.0 as f64 / 10.0
    }

    /// Raw value in tenths of millions
    pub const fn raw(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "£{:.1}M", self.to_millions())
    }
}

impl Default for Cost {
    fn default() -> Self {
        Cost::ZERO
    }
}

impl Add for Cost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Cost(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Cost {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Cost(self.0.saturating_sub(rhs.0))
    }
}

impl Sum for Cost {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Cost(0), |acc, c| acc + c)
    }
}

impl<'a> Sum<&'a Cost> for Cost {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Cost(0), |acc, c| acc + *c)
    }
}

impl From<u16> for Cost {
    fn from(value: u16) -> Self {
        Cost(value)
    }
}

impl From<Cost> for f64 {
    fn from(cost: Cost) -> Self {
        cost.0 as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_millions() {
        assert_eq!(Cost::from_millions(10.0), Cost(100));
        assert_eq!(Cost::from_millions(10.5), Cost(105));
        assert_eq!(Cost::from_millions(4.5), Cost(45));
    }

    #[test]
    fn test_to_millions() {
        assert_eq!(Cost(100).to_millions(), 10.0);
        assert_eq!(Cost(105).to_millions(), 10.5);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Cost(100)), "£10.0M");
        assert_eq!(format!("{}", Cost(127)), "£12.7M");
    }

    #[test]
    fn test_sum() {
        let costs = vec![Cost(50), Cost(60), Cost(45)];
        let total: Cost = costs.iter().sum();
        assert_eq!(total, Cost(155));
    }

    #[test]
    fn test_add() {
        assert_eq!(Cost(50) + Cost(60), Cost(110));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Cost(100) - Cost(40), Cost(60));
    }

    #[test]
    fn test_saturating_sub() {
        assert_eq!(Cost(10) - Cost(20), Cost(0));
    }

    #[test]
    fn test_default() {
        assert_eq!(Cost::default(), Cost::ZERO);
    }

    #[test]
    fn test_budget_default() {
        assert_eq!(Cost::BUDGET_DEFAULT, Cost(1000));
        assert_eq!(Cost::BUDGET_DEFAULT.to_millions(), 100.0);
    }
}
