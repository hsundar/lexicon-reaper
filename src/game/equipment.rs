use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponKind {
    BasicScythe,
    VenomScythe,
    CrystalScythe,
    ThunderScythe,
    VampireScythe,
    GiantScythe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub kind: WeaponKind,
    pub level: u32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            kind: WeaponKind::BasicScythe,
            level: 1,
        }
    }
}

impl Weapon {
    pub fn name(&self) -> &'static str {
        match self.kind {
            WeaponKind::BasicScythe => "Basic Scythe",
            WeaponKind::VenomScythe => "Venom Scythe",
            WeaponKind::CrystalScythe => "Crystal Scythe",
            WeaponKind::ThunderScythe => "Thunder Scythe",
            WeaponKind::VampireScythe => "Vampire Scythe",
            WeaponKind::GiantScythe => "Giant Scythe",
        }
    }

    pub fn damage_bonus(&self) -> f64 {
        let base = match self.kind {
            WeaponKind::BasicScythe => 1.0,
            WeaponKind::VenomScythe => 0.5,
            WeaponKind::CrystalScythe => 0.5,
            WeaponKind::ThunderScythe => 2.0,
            WeaponKind::VampireScythe => 0.5,
            WeaponKind::GiantScythe => 0.0,
        };
        base * self.level as f64
    }

    /// Extra damage for long words (GiantScythe)
    pub fn long_word_bonus(&self, word_len: usize) -> f64 {
        match self.kind {
            WeaponKind::GiantScythe if word_len >= 6 => 3.0 * self.level as f64,
            _ => 0.0,
        }
    }

    /// Lifesteal fraction (VampireScythe)
    pub fn lifesteal_fraction(&self) -> f64 {
        match self.kind {
            WeaponKind::VampireScythe => 0.05 * self.level as f64,
            _ => 0.0,
        }
    }

    pub fn dodge_bonus(&self) -> f64 {
        match self.kind {
            WeaponKind::BasicScythe => 0.002 * self.level as f64,
            _ => 0.0,
        }
    }

    pub fn crit_bonus(&self) -> f64 {
        match self.kind {
            WeaponKind::ThunderScythe => 0.01 * self.level as f64,
            _ => 0.0,
        }
    }

    pub fn upgrade_cost(&self) -> u64 {
        50 * (self.level as u64 + 1) * (self.level as u64 + 1)
    }

    pub fn buy_cost(kind: &WeaponKind) -> u64 {
        match kind {
            WeaponKind::BasicScythe => 0,
            WeaponKind::VenomScythe => 150,
            WeaponKind::CrystalScythe => 200,
            WeaponKind::ThunderScythe => 300,
            WeaponKind::VampireScythe => 250,
            WeaponKind::GiantScythe => 350,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookKind {
    Strength,
    Wisdom,
    Health,
    Speed,
    Purity,
    Wealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub kind: BookKind,
    pub level: u32,
    pub xp: u32,
}

impl Book {
    pub fn new(kind: BookKind) -> Self {
        Self {
            kind,
            level: 1,
            xp: 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            BookKind::Strength => "Book of Strength",
            BookKind::Wisdom => "Book of Wisdom",
            BookKind::Health => "Book of Health",
            BookKind::Speed => "Book of Speed",
            BookKind::Purity => "Book of Purity",
            BookKind::Wealth => "Book of Wealth",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self.kind {
            BookKind::Strength => "STR",
            BookKind::Wisdom => "WIS",
            BookKind::Health => "HP",
            BookKind::Speed => "SPD",
            BookKind::Purity => "PUR",
            BookKind::Wealth => "GEM",
        }
    }

    pub fn xp_to_next_level(&self) -> u32 {
        10 * self.level * self.level
    }

    pub fn add_xp(&mut self, amount: u32) {
        self.xp += amount;
        while self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
        }
    }

    pub fn buy_cost(kind: &BookKind) -> u64 {
        match kind {
            BookKind::Strength => 100,
            BookKind::Wisdom => 120,
            BookKind::Health => 80,
            BookKind::Speed => 150,
            BookKind::Purity => 130,
            BookKind::Wealth => 200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Potion {
    Healing { power: i32 },
    Purity,
    Lifesteal,
}

impl Potion {
    pub fn name(&self) -> &'static str {
        match self {
            Potion::Healing { .. } => "Healing Potion",
            Potion::Purity => "Purity Potion",
            Potion::Lifesteal => "Lifesteal Potion",
        }
    }

    pub fn buy_cost(&self) -> u64 {
        match self {
            Potion::Healing { .. } => 30,
            Potion::Purity => 50,
            Potion::Lifesteal => 40,
        }
    }
}
