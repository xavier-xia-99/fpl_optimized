//! CSV output formatting

use crate::error::Result;
use crate::solver::ProblemResult;
use std::path::Path;

/// Write problem result to CSV file
pub fn write_csv(result: &ProblemResult, path: &Path) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;

    // Write header
    writer.write_record([
        "player_id",
        "web_name",
        "team_code",
        "element_type",
        "now_cost",
        "event",
        "points_md",
        "is_captain",
        "multiplier",
        "starting_lineup",
        "selected_by_percent",
        "gw_points",
    ])?;

    let gw_num = result.gameweek.map(|g| g.raw()).unwrap_or(0);

    for player in &result.squad {
        writer.write_record([
            player.id.raw().to_string(),
            player.web_name.clone(),
            player.team_code.to_string(),
            (player.position as u8).to_string(),
            player.cost.raw().to_string(),
            gw_num.to_string(),
            format!("{:.2}", player.expected_points),
            player.is_captain.to_string(),
            player.multiplier.to_string(),
            (player.is_in_lineup as u8).to_string(),
            format!("{:.1}", player.ownership_percent),
            format!("{:.2}", player.gw_points()),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// Write problem result to CSV string
pub fn to_csv_string(result: &ProblemResult) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());

    // Write header
    writer.write_record([
        "player_id",
        "web_name",
        "team_code",
        "element_type",
        "now_cost",
        "event",
        "points_md",
        "is_captain",
        "multiplier",
        "starting_lineup",
        "selected_by_percent",
        "gw_points",
    ])?;

    let gw_num = result.gameweek.map(|g| g.raw()).unwrap_or(0);

    for player in &result.squad {
        writer.write_record([
            player.id.raw().to_string(),
            player.web_name.clone(),
            player.team_code.to_string(),
            (player.position as u8).to_string(),
            player.cost.raw().to_string(),
            gw_num.to_string(),
            format!("{:.2}", player.expected_points),
            player.is_captain.to_string(),
            player.multiplier.to_string(),
            (player.is_in_lineup as u8).to_string(),
            format!("{:.1}", player.ownership_percent),
            format!("{:.2}", player.gw_points()),
        ])?;
    }

    let bytes = writer.into_inner().map_err(|e| {
        crate::error::FplError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    Ok(String::from_utf8_lossy(&bytes).to_string())
}
