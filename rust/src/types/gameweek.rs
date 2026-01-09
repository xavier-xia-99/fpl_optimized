//! Gameweek type for FPL

use serde::{Deserialize, Serialize};
use std::fmt;

/// Gameweek number (1-38)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Gameweek(pub u8);

impl Gameweek {
    /// Minimum gameweek
    pub const MIN: Gameweek = Gameweek(1);

    /// Maximum gameweek
    pub const MAX: Gameweek = Gameweek(38);

    /// Create a new gameweek
    pub fn new(gw: u8) -> Option<Self> {
        if (1..=38).contains(&gw) {
            Some(Gameweek(gw))
        } else {
            None
        }
    }

    /// Create a gameweek without validation (use with care)
    pub const fn new_unchecked(gw: u8) -> Self {
        Gameweek(gw)
    }

    /// Get the raw gameweek number
    pub const fn raw(&self) -> u8 {
        self.0
    }

    /// Check if this is a valid gameweek
    pub const fn is_valid(&self) -> bool {
        self.0 >= 1 && self.0 <= 38
    }

    /// Get the next gameweek, if valid
    pub fn next(&self) -> Option<Self> {
        if self.0 < 38 {
            Some(Gameweek(self.0 + 1))
        } else {
            None
        }
    }

    /// Get the previous gameweek, if valid
    pub fn prev(&self) -> Option<Self> {
        if self.0 > 1 {
            Some(Gameweek(self.0 - 1))
        } else {
            None
        }
    }

    /// Create a range of gameweeks
    pub fn range(start: u8, end: u8) -> impl Iterator<Item = Gameweek> {
        (start..=end).filter_map(Gameweek::new)
    }
}

impl fmt::Display for Gameweek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GW{}", self.0)
    }
}

impl TryFrom<u8> for Gameweek {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Gameweek::new(value).ok_or("Gameweek must be between 1 and 38")
    }
}

impl From<Gameweek> for u8 {
    fn from(gw: Gameweek) -> Self {
        gw.0
    }
}

impl From<Gameweek> for u32 {
    fn from(gw: Gameweek) -> Self {
        gw.0 as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        assert!(Gameweek::new(1).is_some());
        assert!(Gameweek::new(38).is_some());
        assert!(Gameweek::new(20).is_some());
    }

    #[test]
    fn test_new_invalid() {
        assert!(Gameweek::new(0).is_none());
        assert!(Gameweek::new(39).is_none());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Gameweek(1)), "GW1");
        assert_eq!(format!("{}", Gameweek(21)), "GW21");
    }

    #[test]
    fn test_next() {
        assert_eq!(Gameweek(1).next(), Some(Gameweek(2)));
        assert_eq!(Gameweek(38).next(), None);
    }

    #[test]
    fn test_prev() {
        assert_eq!(Gameweek(2).prev(), Some(Gameweek(1)));
        assert_eq!(Gameweek(1).prev(), None);
    }

    #[test]
    fn test_range() {
        let gws: Vec<_> = Gameweek::range(1, 3).collect();
        assert_eq!(gws, vec![Gameweek(1), Gameweek(2), Gameweek(3)]);
    }
}
