//! JSON output formatting

use crate::error::Result;
use crate::solver::IterativeResult;
use std::path::Path;

/// Write iterative results to JSON file
pub fn write_json(results: &[IterativeResult], path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Convert iterative results to JSON string
pub fn to_json_string(results: &[IterativeResult]) -> Result<String> {
    Ok(serde_json::to_string_pretty(results)?)
}

impl From<serde_json::Error> for crate::error::FplError {
    fn from(err: serde_json::Error) -> Self {
        crate::error::FplError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        ))
    }
}
