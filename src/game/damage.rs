use super::board::Board;
use super::enemy::Enemy;
use super::equipment::BookKind;
use super::player::Player;
use super::tile::TileEffect;

/// Word length multiplier - rewards longer words significantly
fn length_multiplier(word_len: usize) -> f64 {
    match word_len {
        0..=2 => 0.5,
        3 => 1.0,
        4 => 1.3,
        5 => 1.7,
        6 => 2.2,
        7 => 2.8,
        8 => 3.5,
        n => 3.5 + 0.5 * (n as f64 - 8.0),
    }
}

/// Bonus for using uncommon/rare letters (Wisdom book)
fn uncommon_letter_bonus(word: &str, wisdom_level: u32) -> f64 {
    if wisdom_level == 0 {
        return 0.0;
    }
    let bonus_per_level = 0.5;
    let mut count = 0;
    for ch in word.chars() {
        match ch.to_ascii_uppercase() {
            'J' | 'X' | 'Q' | 'Z' | 'K' | 'V' => count += 1,
            _ => {}
        }
    }
    count as f64 * bonus_per_level * wisdom_level as f64
}

pub struct DamageResult {
    pub raw_damage: i32,
    pub is_crit: bool,
    pub self_damage: i32,
    pub lifesteal: i32,
    pub bonus_turn: bool,
}

/// Calculate damage for the current word on the board
pub fn calculate_damage(board: &Board, player: &Player, enemy: &Enemy) -> DamageResult {
    let tile_values = board.selected_tile_values();
    let word_len = tile_values.len();
    let word = board.selected_word();

    if word_len == 0 {
        return DamageResult {
            raw_damage: 0,
            is_crit: false,
            self_damage: 0,
            lifesteal: 0,
            bonus_turn: false,
        };
    }

    let base_value: f64 = tile_values.iter().sum();
    let length_mult = length_multiplier(word_len);
    let weapon_bonus = player.weapon.damage_bonus();
    let long_word_bonus = player.weapon.long_word_bonus(word_len);

    // Book bonuses
    let mut strength_bonus = 0.0;
    let mut wisdom_level = 0u32;
    let mut speed_chance = 0.0;
    for book in &player.books {
        match book.kind {
            BookKind::Strength => strength_bonus += 1.5 * book.level as f64,
            BookKind::Wisdom => wisdom_level += book.level,
            BookKind::Speed => speed_chance += 0.05 * book.level as f64,
            _ => {}
        }
    }

    let wisdom_bonus = uncommon_letter_bonus(&word, wisdom_level);

    let mut raw = (base_value * length_mult) + player.base_attack as f64 + weapon_bonus
        + long_word_bonus
        + strength_bonus
        + wisdom_bonus;

    // Crit check
    let crit_chance = player.crit_chance + player.weapon.crit_bonus();
    let is_crit = rand::random::<f64>() < crit_chance;
    if is_crit {
        raw *= player.crit_multiplier;
    }

    let final_damage = (raw as i32 - enemy.defense).max(1);

    // Self-damage from poison tiles (20% of dealt damage per poison tile)
    let poison_count = board.count_selected_with_effect(TileEffect::Poison);
    let poison_self_damage = (final_damage as f64 * 0.20 * poison_count as f64) as i32;

    // Self-damage from spike tiles (flat 8 HP per spike tile)
    let spike_count = board.count_selected_with_effect(TileEffect::Spike);
    let spike_self_damage = 8 * spike_count as i32;

    let total_self_damage = poison_self_damage + spike_self_damage;

    // Lifesteal
    let lifesteal_frac = player.weapon.lifesteal_fraction();
    let lifesteal = (final_damage as f64 * lifesteal_frac) as i32;

    // Bonus turn from Speed book
    let bonus_turn = speed_chance > 0.0 && rand::random::<f64>() < speed_chance.min(0.30);

    DamageResult {
        raw_damage: final_damage,
        is_crit,
        self_damage: total_self_damage,
        lifesteal,
        bonus_turn,
    }
}

/// Estimate damage for display (without crit randomness)
pub fn estimate_damage(board: &Board, player: &Player, enemy: &Enemy) -> i32 {
    let tile_values = board.selected_tile_values();
    let word_len = tile_values.len();
    let word = board.selected_word();
    if word_len == 0 {
        return 0;
    }

    let base_value: f64 = tile_values.iter().sum();
    let length_mult = length_multiplier(word_len);
    let weapon_bonus = player.weapon.damage_bonus();
    let long_word_bonus = player.weapon.long_word_bonus(word_len);

    let mut strength_bonus = 0.0;
    let mut wisdom_level = 0u32;

    for book in &player.books {
        match book.kind {
            BookKind::Strength => strength_bonus += 1.5 * book.level as f64,
            BookKind::Wisdom => wisdom_level += book.level,
            _ => {}
        }
    }

    let wisdom_bonus = uncommon_letter_bonus(&word, wisdom_level);

    let raw = (base_value * length_mult) + player.base_attack as f64 + weapon_bonus
        + long_word_bonus
        + strength_bonus
        + wisdom_bonus;

    (raw as i32 - enemy.defense).max(1)
}
