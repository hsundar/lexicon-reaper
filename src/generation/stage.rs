use rand::Rng;

use crate::game::enemy::Enemy;

use super::enemy_gen::generate_enemy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Forest,
    Crypt,
    Volcano,
    Abyss,
    Void,
}

impl Biome {
    pub fn from_stage(stage: u32) -> Self {
        match ((stage - 1) / 10) % 5 {
            0 => Biome::Forest,
            1 => Biome::Crypt,
            2 => Biome::Volcano,
            3 => Biome::Abyss,
            _ => Biome::Void,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Biome::Forest => "Haunted Forest",
            Biome::Crypt => "Ancient Crypt",
            Biome::Volcano => "Molten Cavern",
            Biome::Abyss => "The Abyss",
            Biome::Void => "The Void",
        }
    }
}

pub struct StageConfig {
    pub enemies: Vec<Enemy>,
    pub biome: Biome,
}

pub fn generate_stage<R: Rng>(rng: &mut R, stage: u32) -> StageConfig {
    let biome = Biome::from_stage(stage);
    let is_boss_stage = stage % 10 == 0;

    let enemies = if is_boss_stage {
        vec![generate_enemy(rng, stage, biome, true)]
    } else {
        let enemy_count = if stage <= 3 {
            1
        } else if stage <= 10 {
            rng.gen_range(1..=2)
        } else {
            rng.gen_range(1..=3)
        };
        (0..enemy_count)
            .map(|_| generate_enemy(rng, stage, biome, false))
            .collect()
    };

    StageConfig {
        enemies,
        biome,
    }
}
