//! Team types for FPL

use serde::{Deserialize, Serialize};
use std::fmt;

/// Team code identifier from FPL API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamCode(pub u8);

impl TeamCode {
    /// Create a new team code
    pub const fn new(code: u8) -> Self {
        TeamCode(code)
    }

    /// Get the raw code value
    pub const fn raw(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for TeamCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for TeamCode {
    fn from(code: u8) -> Self {
        TeamCode(code)
    }
}

impl From<TeamCode> for u8 {
    fn from(code: TeamCode) -> Self {
        code.0
    }
}

/// A Premier League team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Team code (unique identifier)
    pub code: TeamCode,
    /// Team ID in FPL API
    pub id: u8,
    /// Short name (e.g., "ARS", "CHE")
    pub short_name: String,
    /// Full name (e.g., "Arsenal", "Chelsea")
    pub name: String,
}

impl Team {
    /// Create a new team
    pub fn new(code: u8, id: u8, short_name: impl Into<String>, name: impl Into<String>) -> Self {
        Team {
            code: TeamCode(code),
            id,
            short_name: short_name.into(),
            name: name.into(),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_code_new() {
        let code = TeamCode::new(3);
        assert_eq!(code.raw(), 3);
    }

    #[test]
    fn test_team_code_from() {
        let code: TeamCode = 14.into();
        assert_eq!(code.0, 14);
    }

    #[test]
    fn test_team_display() {
        let team = Team::new(3, 1, "ARS", "Arsenal");
        assert_eq!(format!("{}", team), "ARS");
    }
}
