use serde::{Deserialize, Serialize};

use super::tile::TileEffect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub is_boss: bool,
    pub sprite_key: String,
    pub word_constraint: Option<WordConstraint>,
    pub abilities: Vec<EnemyAbility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WordConstraint {
    MinLength(usize),
    MaxLength(usize),
    MustStartWithVowel,
    MustContainLetter(char),
    NoRepeatedLetters,
}

impl WordConstraint {
    pub fn check(&self, word: &str) -> bool {
        let word_upper = word.to_uppercase();
        match self {
            WordConstraint::MinLength(n) => word_upper.len() >= *n,
            WordConstraint::MaxLength(n) => word_upper.len() <= *n,
            WordConstraint::MustStartWithVowel => {
                word_upper
                    .chars()
                    .next()
                    .map(|c| "AEIOU".contains(c))
                    .unwrap_or(false)
            }
            WordConstraint::MustContainLetter(ch) => word_upper.contains(ch.to_ascii_uppercase()),
            WordConstraint::NoRepeatedLetters => {
                let chars: Vec<char> = word_upper.chars().collect();
                let unique: std::collections::HashSet<char> = chars.iter().copied().collect();
                chars.len() == unique.len()
            }
        }
    }

    pub fn description(&self) -> String {
        match self {
            WordConstraint::MinLength(n) => format!("{n}+ letter words only!"),
            WordConstraint::MaxLength(n) => format!("{n} letters max!"),
            WordConstraint::MustStartWithVowel => "Must start with a vowel!".to_string(),
            WordConstraint::MustContainLetter(ch) => {
                format!("Must contain '{}'!", ch.to_ascii_uppercase())
            }
            WordConstraint::NoRepeatedLetters => "No repeated letters!".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyAbility {
    ApplyEffect {
        effect: TileEffect,
        count: usize,
        timer: u8,
    },
    HealSelf {
        amount: i32,
    },
    Enrage {
        bonus_attack: i32,
    },
}

impl Enemy {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn take_damage(&mut self, damage: i32) -> i32 {
        let actual = (damage - self.defense).max(1);
        self.hp = (self.hp - actual).max(0);
        actual
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}

/// Simple test enemy for Phase 1
pub fn test_enemy() -> Enemy {
    Enemy {
        name: "Forest Goblin".to_string(),
        hp: 50,
        max_hp: 50,
        attack: 8,
        defense: 0,
        is_boss: false,
        sprite_key: "goblin".to_string(),
        word_constraint: None,
        abilities: vec![],
    }
}
