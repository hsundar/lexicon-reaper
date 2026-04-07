use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunStats {
    pub words_submitted: u32,
    pub total_damage_dealt: u64,
    pub total_damage_taken: u64,
    pub enemies_defeated: u32,
    pub bosses_defeated: u32,
    pub crits_landed: u32,
    pub dodges: u32,
    pub longest_word: String,
    pub highest_damage_word: String,
    pub highest_damage: i32,
    pub potions_used: u32,
    pub gems_earned: u64,
    pub gems_spent: u64,
    pub tiles_shuffled: u32,
    pub word_history: Vec<String>,
}

impl RunStats {
    pub fn record_word(&mut self, word: &str, damage: i32, is_crit: bool) {
        self.words_submitted += 1;
        self.total_damage_dealt += damage as u64;

        if word.len() > self.longest_word.len() {
            self.longest_word = word.to_string();
        }

        if damage > self.highest_damage {
            self.highest_damage = damage;
            self.highest_damage_word = word.to_string();
        }

        if is_crit {
            self.crits_landed += 1;
        }

        // Keep last 20 words
        self.word_history.push(word.to_string());
        if self.word_history.len() > 20 {
            self.word_history.remove(0);
        }
    }

    pub fn record_enemy_defeated(&mut self, is_boss: bool) {
        self.enemies_defeated += 1;
        if is_boss {
            self.bosses_defeated += 1;
        }
    }

    pub fn record_damage_taken(&mut self, damage: i32) {
        self.total_damage_taken += damage as u64;
    }

    pub fn record_dodge(&mut self) {
        self.dodges += 1;
    }
}
