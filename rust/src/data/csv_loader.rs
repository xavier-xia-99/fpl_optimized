//! CSV data loader for FPL optimization

use crate::error::{DataError, FplError, Result};
use crate::types::{Cost, Gameweek, Player, PlayerId, PlayerGameweekProjection, Position, ProjectionData, TeamCode};
use std::collections::HashMap;
use std::path::Path;

/// Loader for CSV data files
pub struct CsvLoader;

impl CsvLoader {
    /// Load player data from element.csv
    pub fn load_players(path: &Path) -> Result<Vec<Player>> {
        let file_path = path.join("element.csv");
        let mut rdr = csv::Reader::from_path(&file_path)
            .map_err(|e| FplError::Data(DataError::CsvParse(format!("{}: {}", file_path.display(), e))))?;

        let mut players = Vec::new();

        for result in rdr.deserialize() {
            let record: PlayerRecord = result
                .map_err(|e| FplError::Data(DataError::CsvParse(e.to_string())))?;

            let position = Position::try_from(record.element_type)
                .map_err(|e| FplError::Data(DataError::InvalidFormat(e.to_string())))?;

            let player = Player {
                id: PlayerId(record.id),
                web_name: record.web_name,
                full_name: Some(format!("{} {}", record.first_name, record.second_name)),
                team_code: TeamCode(record.team_code),
                position,
                cost: Cost(record.now_cost),
                ownership_percent: record.selected_by_percent,
                form: record.form.and_then(|s| s.parse().ok()),
                ict_index: record.ict_index.and_then(|s| s.parse().ok()),
                ep_this: record.ep_this.and_then(|s| s.parse().ok()),
                points_per_game: record.points_per_game.and_then(|s| s.parse().ok()),
                bps: Some(record.bps as f64),
                influence: record.influence.and_then(|s| s.parse().ok()),
                creativity: record.creativity.and_then(|s| s.parse().ok()),
                threat: record.threat.and_then(|s| s.parse().ok()),
            };

            players.push(player);
        }

        if players.is_empty() {
            return Err(FplError::Data(DataError::NoPlayersLoaded));
        }

        Ok(players)
    }

    /// Load gameweek projections from element_gameweek.csv
    pub fn load_projections(path: &Path) -> Result<ProjectionData> {
        let file_path = path.join("element_gameweek.csv");
        let mut rdr = csv::Reader::from_path(&file_path)
            .map_err(|e| FplError::Data(DataError::CsvParse(format!("{}: {}", file_path.display(), e))))?;

        let mut projections = ProjectionData::new();

        for result in rdr.deserialize() {
            let record: ProjectionRecord = result
                .map_err(|e| FplError::Data(DataError::CsvParse(e.to_string())))?;

            let projection = PlayerGameweekProjection::with_details(
                PlayerId(record.player_id),
                Gameweek::new_unchecked(record.event),
                record.points_md,
                record.xmins_md,
                record.opponent_team.map(TeamCode),
                record.was_home,
            );

            projections.insert(projection);
        }

        if projections.is_empty() {
            return Err(FplError::Data(DataError::NoProjectionsLoaded));
        }

        Ok(projections)
    }

    /// Load element type data (position constraints)
    pub fn load_element_types(path: &Path) -> Result<HashMap<u8, ElementTypeData>> {
        let file_path = path.join("element_type.csv");
        let mut rdr = csv::Reader::from_path(&file_path)
            .map_err(|e| FplError::Data(DataError::CsvParse(format!("{}: {}", file_path.display(), e))))?;

        let mut types = HashMap::new();

        for result in rdr.deserialize() {
            let record: ElementTypeRecord = result
                .map_err(|e| FplError::Data(DataError::CsvParse(e.to_string())))?;

            types.insert(
                record.id,
                ElementTypeData {
                    id: record.id,
                    singular_name: record.singular_name,
                    singular_name_short: record.singular_name_short,
                    squad_select: record.squad_select,
                    squad_min_play: record.squad_min_play,
                    squad_max_play: record.squad_max_play,
                },
            );
        }

        Ok(types)
    }
}

/// Raw player record from CSV
#[derive(Debug, serde::Deserialize)]
struct PlayerRecord {
    id: u32,
    first_name: String,
    second_name: String,
    web_name: String,
    team_code: u8,
    element_type: u8,
    now_cost: u16,
    selected_by_percent: f64,
    form: Option<String>,
    ict_index: Option<String>,
    ep_this: Option<String>,
    points_per_game: Option<String>,
    #[serde(default)]
    bps: i32,
    influence: Option<String>,
    creativity: Option<String>,
    threat: Option<String>,
}

/// Raw projection record from CSV
#[derive(Debug, serde::Deserialize)]
struct ProjectionRecord {
    player_id: u32,
    event: u8,
    points_md: f64,
    #[serde(default)]
    xmins_md: f64,
    #[serde(default)]
    opponent_team: Option<u8>,
    #[serde(default)]
    was_home: Option<bool>,
}

/// Element type data from CSV
#[derive(Debug, Clone)]
pub struct ElementTypeData {
    pub id: u8,
    pub singular_name: String,
    pub singular_name_short: String,
    pub squad_select: u8,
    pub squad_min_play: u8,
    pub squad_max_play: u8,
}

/// Raw element type record from CSV
#[derive(Debug, serde::Deserialize)]
struct ElementTypeRecord {
    id: u8,
    singular_name: String,
    singular_name_short: String,
    squad_select: u8,
    squad_min_play: u8,
    squad_max_play: u8,
}
