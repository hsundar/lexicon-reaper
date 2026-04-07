use serde::{Deserialize, Serialize};

use super::equipment::{Book, Potion, Weapon};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub hp: i32,
    pub max_hp: i32,
    pub base_attack: i32,
    pub armor: i32,
    pub dodge_chance: f64,
    pub crit_chance: f64,
    pub crit_multiplier: f64,
    pub gems: u64,
    pub weapon: Weapon,
    pub books: Vec<Book>,
    pub potions: Vec<Potion>,
    pub max_potions: usize,
    pub upgrades: PlayerUpgrades,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerUpgrades {
    pub hp_level: u32,
    pub attack_level: u32,
    pub armor_level: u32,
    pub dodge_level: u32,
    pub potion_capacity_level: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            hp: 100,
            max_hp: 100,
            base_attack: 5,
            armor: 0,
            dodge_chance: 0.05,
            crit_chance: 0.10,
            crit_multiplier: 1.5,
            gems: 0,
            weapon: Weapon::default(),
            books: Vec::new(),
            potions: vec![Potion::Healing { power: 30 }],
            max_potions: 3,
            upgrades: PlayerUpgrades::default(),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn take_damage(&mut self, raw_damage: i32) -> i32 {
        let actual = (raw_damage - self.armor).max(1);
        self.hp = (self.hp - actual).max(0);
        actual
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn use_potion(&mut self, index: usize) -> Option<String> {
        if index >= self.potions.len() {
            return None;
        }
        let potion = self.potions.remove(index);
        match potion {
            Potion::Healing { power } => {
                self.heal(power);
                Some(format!("Healed for {} HP!", power))
            }
            Potion::Purity => Some("Cleansed all tile effects!".to_string()),
            Potion::Lifesteal => Some("Next word heals for damage dealt!".to_string()),
        }
    }

    pub fn has_potions(&self) -> bool {
        !self.potions.is_empty()
    }
}
