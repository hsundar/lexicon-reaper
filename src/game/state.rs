use serde::{Deserialize, Serialize};

use super::board::Board;
use super::enemy::Enemy;
use super::shop::ShopItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub board: Board,
    pub enemies: Vec<Enemy>,
    pub current_enemy_idx: usize,
    pub turn_count: u32,
    pub cursor: (usize, usize),
    pub phase: CombatPhase,
    pub lifesteal_active: bool,
}

impl CombatState {
    pub fn current_enemy(&self) -> &Enemy {
        &self.enemies[self.current_enemy_idx]
    }

    pub fn current_enemy_mut(&mut self) -> &mut Enemy {
        &mut self.enemies[self.current_enemy_idx]
    }

    pub fn has_next_enemy(&self) -> bool {
        self.current_enemy_idx + 1 < self.enemies.len()
    }

    pub fn advance_enemy(&mut self) -> bool {
        if self.has_next_enemy() {
            self.current_enemy_idx += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatPhase {
    PlayerInput,
    PlayerAttackAnim {
        damage: i32,
        is_crit: bool,
        word: String,
        ticks_remaining: u8,
    },
    EnemyTurn {
        ticks_remaining: u8,
    },
    EnemyHitAnim {
        damage: i32,
        dodged: bool,
        ticks_remaining: u8,
    },
    EnemyDefeated {
        ticks_remaining: u8,
    },
    Walking {
        ticks_remaining: u8,
    },
    StageVictory {
        gems_earned: u64,
        ticks_remaining: u8,
    },
    Defeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopState {
    pub items: Vec<(ShopItem, u64)>,
    pub selected_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Combat,
    Map,
    Shop,
    Inventory,
    GameOver,
}

pub struct LogEntry {
    pub text: String,
    pub color: LogColor,
}

#[derive(Debug, Clone, Copy)]
pub enum LogColor {
    Normal,
    PlayerDamage,
    EnemyDamage,
    Heal,
    Crit,
    Info,
    Warning,
}
