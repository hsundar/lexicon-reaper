use rand::seq::SliceRandom;
use rand::Rng;

use crate::game::enemy::{Enemy, EnemyAbility, WordConstraint};
use crate::game::tile::TileEffect;

use super::stage::Biome;

struct EnemyTemplate {
    name: &'static str,
    sprite: &'static str,
}

const FOREST_ENEMIES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Forest Goblin", sprite: "goblin" },
    EnemyTemplate { name: "Wild Wolf", sprite: "wolf" },
    EnemyTemplate { name: "Toxic Slime", sprite: "slime" },
    EnemyTemplate { name: "Cave Bat", sprite: "goblin" },
    EnemyTemplate { name: "Thorn Sprite", sprite: "slime" },
];

const CRYPT_ENEMIES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Skeleton Guard", sprite: "skeleton" },
    EnemyTemplate { name: "Bone Rattler", sprite: "skeleton" },
    EnemyTemplate { name: "Phantom", sprite: "goblin" },
    EnemyTemplate { name: "Crypt Spider", sprite: "slime" },
    EnemyTemplate { name: "Wraith", sprite: "skeleton" },
];

const VOLCANO_ENEMIES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Fire Imp", sprite: "goblin" },
    EnemyTemplate { name: "Lava Slime", sprite: "slime" },
    EnemyTemplate { name: "Magma Golem", sprite: "skeleton" },
    EnemyTemplate { name: "Ember Wolf", sprite: "wolf" },
    EnemyTemplate { name: "Flame Dancer", sprite: "goblin" },
];

const ABYSS_ENEMIES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Shadow Demon", sprite: "skeleton" },
    EnemyTemplate { name: "Void Walker", sprite: "goblin" },
    EnemyTemplate { name: "Abyssal Leech", sprite: "slime" },
    EnemyTemplate { name: "Dark Stalker", sprite: "wolf" },
    EnemyTemplate { name: "Nightmare", sprite: "skeleton" },
];

const VOID_ENEMIES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Chaos Spawn", sprite: "slime" },
    EnemyTemplate { name: "Reality Shredder", sprite: "skeleton" },
    EnemyTemplate { name: "Entropy Beast", sprite: "wolf" },
    EnemyTemplate { name: "Null Wraith", sprite: "goblin" },
    EnemyTemplate { name: "Void Titan", sprite: "skeleton" },
];

const FOREST_BOSSES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Elder Treant", sprite: "boss_goblin" },
    EnemyTemplate { name: "Goblin King", sprite: "boss_goblin" },
];

const CRYPT_BOSSES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Lich Lord", sprite: "boss_goblin" },
    EnemyTemplate { name: "Death Knight", sprite: "boss_goblin" },
];

const VOLCANO_BOSSES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Inferno Dragon", sprite: "boss_goblin" },
    EnemyTemplate { name: "Molten King", sprite: "boss_goblin" },
];

const ABYSS_BOSSES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "Demon Lord", sprite: "boss_goblin" },
    EnemyTemplate { name: "Shadow Monarch", sprite: "boss_goblin" },
];

const VOID_BOSSES: &[EnemyTemplate] = &[
    EnemyTemplate { name: "The Unmaker", sprite: "boss_goblin" },
    EnemyTemplate { name: "Entropy Incarnate", sprite: "boss_goblin" },
];

fn templates_for_biome(biome: Biome, is_boss: bool) -> &'static [EnemyTemplate] {
    if is_boss {
        match biome {
            Biome::Forest => FOREST_BOSSES,
            Biome::Crypt => CRYPT_BOSSES,
            Biome::Volcano => VOLCANO_BOSSES,
            Biome::Abyss => ABYSS_BOSSES,
            Biome::Void => VOID_BOSSES,
        }
    } else {
        match biome {
            Biome::Forest => FOREST_ENEMIES,
            Biome::Crypt => CRYPT_ENEMIES,
            Biome::Volcano => VOLCANO_ENEMIES,
            Biome::Abyss => ABYSS_ENEMIES,
            Biome::Void => VOID_ENEMIES,
        }
    }
}

pub fn generate_enemy<R: Rng>(rng: &mut R, stage: u32, biome: Biome, is_boss: bool) -> Enemy {
    let templates = templates_for_biome(biome, is_boss);
    let template = templates.choose(rng).unwrap();

    let boss_hp_mult = if is_boss { 3 } else { 1 };
    let boss_atk_mult = if is_boss { 2 } else { 1 };

    let hp = (30 + stage as i32 * 8) * boss_hp_mult;
    let attack = (5 + stage as i32 / 2) * boss_atk_mult;
    let defense = (stage as i32 / 10).min(8);

    let abilities = generate_abilities(rng, stage, biome, is_boss);
    let constraint = generate_constraint(rng, stage);

    Enemy {
        name: template.name.to_string(),
        hp,
        max_hp: hp,
        attack,
        defense,
        is_boss,
        sprite_key: template.sprite.to_string(),
        word_constraint: constraint,
        abilities,
    }
}

fn generate_abilities<R: Rng>(
    rng: &mut R,
    stage: u32,
    biome: Biome,
    is_boss: bool,
) -> Vec<EnemyAbility> {
    let mut abilities = Vec::new();
    let max_abilities = if is_boss {
        ((stage / 5) + 2).min(4) as usize
    } else {
        ((stage / 5)).min(3) as usize
    };

    if max_abilities == 0 {
        return abilities;
    }

    // Biome-favored tile effects
    let biome_effects: &[(TileEffect, u32)] = match biome {
        Biome::Forest => &[(TileEffect::Poison, 3), (TileEffect::Whirlwind, 2)],
        Biome::Crypt => &[(TileEffect::Plague, 3), (TileEffect::Broken, 3)],
        Biome::Volcano => &[(TileEffect::Spike, 4), (TileEffect::Stone, 2)],
        Biome::Abyss => &[(TileEffect::Plague, 2), (TileEffect::Duplicator, 2), (TileEffect::Flipped, 2)],
        Biome::Void => &[(TileEffect::Poison, 2), (TileEffect::Spike, 2), (TileEffect::Plague, 2), (TileEffect::Stone, 2)],
    };

    // Add tile effect abilities based on biome
    for (effect, _weight) in biome_effects {
        if abilities.len() >= max_abilities {
            break;
        }
        if rng.gen_bool(0.6) {
            let count = rng.gen_range(1..=2);
            let timer = rng.gen_range(2..=4);
            abilities.push(EnemyAbility::ApplyEffect {
                effect: *effect,
                count,
                timer,
            });
        }
    }

    // Boss gets heal
    if is_boss && abilities.len() < max_abilities {
        abilities.push(EnemyAbility::HealSelf {
            amount: 10 + stage as i32,
        });
    }

    // Chance of enrage at higher stages
    if stage >= 15 && abilities.len() < max_abilities && rng.gen_bool(0.3) {
        abilities.push(EnemyAbility::Enrage {
            bonus_attack: rng.gen_range(1..=3),
        });
    }

    abilities
}

fn generate_constraint<R: Rng>(rng: &mut R, stage: u32) -> Option<WordConstraint> {
    if stage < 5 {
        return None;
    }

    // Probability increases with stage
    let chance = ((stage as f64 - 4.0) * 0.05).min(0.5);
    if !rng.gen_bool(chance) {
        return None;
    }

    let constraints = [
        WordConstraint::MinLength(4),
        WordConstraint::MustStartWithVowel,
        WordConstraint::NoRepeatedLetters,
    ];

    // Higher stages get harder constraints
    let mut pool: Vec<WordConstraint> = constraints.to_vec();
    if stage >= 10 {
        pool.push(WordConstraint::MinLength(5));
    }
    if stage >= 15 {
        let vowels = ['A', 'E', 'I', 'O', 'U'];
        let letter = vowels[rng.gen_range(0..5)];
        pool.push(WordConstraint::MustContainLetter(letter));
    }

    pool.choose(rng).cloned()
}
