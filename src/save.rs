use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::game::player::Player;
use crate::game::stats::RunStats;

const SAVE_VERSION: u32 = 2;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub player: Player,
    pub stage_number: u32,
    pub rng_seed: u64,
    pub stats: RunStats,
}

fn save_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("lexicon-reaper").join("save.json"))
}

pub fn save_game(player: &Player, stage_number: u32, stats: &RunStats) -> Result<(), String> {
    let path = save_path().ok_or("Could not determine save directory")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create save dir: {}", e))?;
    }

    let save = SaveData {
        version: SAVE_VERSION,
        player: player.clone(),
        stage_number,
        rng_seed: rand::random(),
        stats: stats.clone(),
    };

    let json =
        serde_json::to_string_pretty(&save).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write save: {}", e))?;

    Ok(())
}

pub fn load_game() -> Result<Option<SaveData>, String> {
    let path = match save_path() {
        Some(p) => p,
        None => return Ok(None),
    };

    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to read save: {}", e))?;
    let save: SaveData =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse save: {}", e))?;

    Ok(Some(save))
}

pub fn has_save() -> bool {
    save_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn delete_save() -> Result<(), String> {
    let path = match save_path() {
        Some(p) => p,
        None => return Ok(()),
    };
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete save: {}", e))?;
    }
    Ok(())
}
