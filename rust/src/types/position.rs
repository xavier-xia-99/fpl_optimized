//! FPL position types with constraints

use serde::{Deserialize, Serialize};
use std::fmt;

/// Fantasy Premier League positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Position {
    Goalkeeper = 1,
    Defender = 2,
    Midfielder = 3,
    Forward = 4,
}

impl Position {
    /// All positions in order
    pub const ALL: [Position; 4] = [
        Position::Goalkeeper,
        Position::Defender,
        Position::Midfielder,
        Position::Forward,
    ];

    /// Iterator over all positions
    pub fn all() -> impl Iterator<Item = Position> {
        Self::ALL.iter().copied()
    }

    /// Number of this position required in squad (total = 15)
    pub const fn squad_select(&self) -> u8 {
        match self {
            Position::Goalkeeper => 2,
            Position::Defender => 5,
            Position::Midfielder => 5,
            Position::Forward => 3,
        }
    }

    /// Minimum starters for this position
    pub const fn min_play(&self) -> u8 {
        match self {
            Position::Goalkeeper => 1,
            Position::Defender => 3,
            Position::Midfielder => 2,
            Position::Forward => 1,
        }
    }

    /// Maximum starters for this position
    pub const fn max_play(&self) -> u8 {
        match self {
            Position::Goalkeeper => 1,
            Position::Defender => 5,
            Position::Midfielder => 5,
            Position::Forward => 3,
        }
    }

    /// Short name (e.g., "GK", "DEF")
    pub const fn short_name(&self) -> &'static str {
        match self {
            Position::Goalkeeper => "GK",
            Position::Defender => "DEF",
            Position::Midfielder => "MID",
            Position::Forward => "FWD",
        }
    }

    /// Full name
    pub const fn full_name(&self) -> &'static str {
        match self {
            Position::Goalkeeper => "Goalkeeper",
            Position::Defender => "Defender",
            Position::Midfielder => "Midfielder",
            Position::Forward => "Forward",
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

impl TryFrom<u8> for Position {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Position::Goalkeeper),
            2 => Ok(Position::Defender),
            3 => Ok(Position::Midfielder),
            4 => Ok(Position::Forward),
            _ => Err("Invalid position ID (must be 1-4)"),
        }
    }
}

impl From<Position> for u8 {
    fn from(pos: Position) -> Self {
        pos as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squad_totals_15() {
        let total: u8 = Position::all().map(|p| p.squad_select()).sum();
        assert_eq!(total, 15);
    }

    #[test]
    fn test_min_play_totals_7() {
        let total: u8 = Position::all().map(|p| p.min_play()).sum();
        assert_eq!(total, 7);
    }

    #[test]
    fn test_max_play_totals_14() {
        let total: u8 = Position::all().map(|p| p.max_play()).sum();
        assert_eq!(total, 14);
    }

    #[test]
    fn test_try_from_valid() {
        assert_eq!(Position::try_from(1), Ok(Position::Goalkeeper));
        assert_eq!(Position::try_from(2), Ok(Position::Defender));
        assert_eq!(Position::try_from(3), Ok(Position::Midfielder));
        assert_eq!(Position::try_from(4), Ok(Position::Forward));
    }

    #[test]
    fn test_try_from_invalid() {
        assert!(Position::try_from(0).is_err());
        assert!(Position::try_from(5).is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Position::Goalkeeper), "GK");
        assert_eq!(format!("{}", Position::Defender), "DEF");
        assert_eq!(format!("{}", Position::Midfielder), "MID");
        assert_eq!(format!("{}", Position::Forward), "FWD");
    }
}
