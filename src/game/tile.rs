use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileRarity {
    Bronze,
    Silver,
    Gold,
}

impl TileRarity {
    pub fn multiplier(self) -> f64 {
        match self {
            TileRarity::Bronze => 1.0,
            TileRarity::Silver => 2.0,
            TileRarity::Gold => 3.0,
        }
    }

    pub fn dots(self) -> u8 {
        match self {
            TileRarity::Bronze => 1,
            TileRarity::Silver => 2,
            TileRarity::Gold => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileEffect {
    Normal,
    Crystal,
    Poison,
    Spike,
    Plague,
    Stone,
    Whirlwind,
    Duplicator,
    Flipped,
    Broken,
}

impl TileEffect {
    pub fn symbol(self) -> &'static str {
        match self {
            TileEffect::Normal => " ",
            TileEffect::Crystal => "*",
            TileEffect::Poison => "~",
            TileEffect::Spike => "!",
            TileEffect::Plague => "☠",
            TileEffect::Stone => "#",
            TileEffect::Whirlwind => "⟳",
            TileEffect::Duplicator => "◈",
            TileEffect::Flipped => "⇅",
            TileEffect::Broken => "�crack",
        }
    }

    pub fn short_symbol(self) -> char {
        match self {
            TileEffect::Normal => ' ',
            TileEffect::Crystal => '*',
            TileEffect::Poison => '~',
            TileEffect::Spike => '!',
            TileEffect::Plague => '☠',
            TileEffect::Stone => '#',
            TileEffect::Whirlwind => '⟳',
            TileEffect::Duplicator => '◈',
            TileEffect::Flipped => '⇅',
            TileEffect::Broken => '×',
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub letter: char,
    pub rarity: TileRarity,
    pub effect: TileEffect,
    pub timer: Option<u8>,
    pub selected: bool,
    pub select_order: Option<u8>,
}

impl Tile {
    pub fn new(letter: char, rarity: TileRarity) -> Self {
        Self {
            letter,
            rarity,
            effect: TileEffect::Normal,
            timer: None,
            selected: false,
            select_order: None,
        }
    }

    pub fn base_value(&self) -> i32 {
        letter_value(self.letter)
    }

    pub fn effective_value(&self) -> f64 {
        let base = self.base_value() as f64 * self.rarity.multiplier();
        match self.effect {
            TileEffect::Crystal => base * 1.5,
            TileEffect::Broken | TileEffect::Plague => 0.0,
            _ => base,
        }
    }

    pub fn is_selectable(&self) -> bool {
        self.effect != TileEffect::Stone
    }
}

/// Base point value for a letter (Scrabble-inspired)
pub fn letter_value(ch: char) -> i32 {
    match ch.to_ascii_uppercase() {
        'A' | 'E' | 'I' | 'O' | 'N' | 'R' | 'S' | 'T' | 'L' | 'U' => 1,
        'D' | 'G' => 2,
        'B' | 'C' | 'M' | 'P' => 3,
        'F' | 'H' | 'V' | 'W' | 'Y' => 4,
        'K' => 5,
        'J' | 'X' => 8,
        'Q' | 'Z' => 10,
        _ => 1,
    }
}

/// Returns the rarity for a given letter
pub fn letter_rarity(ch: char) -> TileRarity {
    match ch.to_ascii_uppercase() {
        'E' | 'A' | 'I' | 'O' | 'N' | 'R' | 'T' | 'L' | 'S' | 'U' => TileRarity::Bronze,
        'D' | 'G' | 'B' | 'C' | 'M' | 'P' | 'F' | 'H' | 'W' | 'Y' => TileRarity::Silver,
        'K' | 'V' | 'J' | 'X' | 'Q' | 'Z' => TileRarity::Gold,
        _ => TileRarity::Bronze,
    }
}

/// Weighted letter frequency table for random tile generation.
/// Weights are roughly proportional to English letter frequency.
pub const LETTER_WEIGHTS: [(char, u32); 26] = [
    ('A', 82), ('B', 15), ('C', 28), ('D', 43),
    ('E', 127), ('F', 22), ('G', 20), ('H', 61),
    ('I', 70), ('J', 2), ('K', 8), ('L', 40),
    ('M', 24), ('N', 67), ('O', 75), ('P', 19),
    ('Q', 1), ('R', 60), ('S', 63), ('T', 91),
    ('U', 28), ('V', 10), ('W', 24), ('X', 2),
    ('Y', 20), ('Z', 1),
];

/// Generate a random letter based on English frequency weights
pub fn random_letter<R: Rng>(rng: &mut R) -> char {
    let total: u32 = LETTER_WEIGHTS.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(0..total);
    for (letter, weight) in &LETTER_WEIGHTS {
        if roll < *weight {
            return *letter;
        }
        roll -= weight;
    }
    'E' // fallback
}

/// Generate a random tile
pub fn random_tile<R: Rng>(rng: &mut R) -> Tile {
    let letter = random_letter(rng);
    let rarity = letter_rarity(letter);
    Tile::new(letter, rarity)
}
