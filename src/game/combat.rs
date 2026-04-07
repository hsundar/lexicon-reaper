use rand::Rng;

use super::board::Board;
use super::damage::{calculate_damage, DamageResult};
use super::enemy::{Enemy, EnemyAbility};
use super::equipment::BookKind;
use super::player::Player;

pub struct CombatResult {
    pub damage_dealt: i32,
    pub is_crit: bool,
    pub self_damage: i32,
    pub lifesteal: i32,
    pub potion_lifesteal: i32,
    pub word: String,
    pub enemy_defeated: bool,
    pub bonus_turn: bool,
    pub purity_cleansed: bool,
    pub venom_bonus: i32,
}

/// Resolve the player submitting a word
pub fn resolve_player_word(
    board: &mut Board,
    player: &mut Player,
    enemy: &mut Enemy,
    rng: &mut impl Rng,
    lifesteal_potion_active: bool,
) -> CombatResult {
    let word = board.selected_word();

    // VenomScythe: bonus damage per poison tile used
    let venom_bonus = match player.weapon.kind {
        super::equipment::WeaponKind::VenomScythe => {
            let poison_count = board.count_selected_with_effect(super::tile::TileEffect::Poison);
            (3.0 * player.weapon.level as f64 * poison_count as f64) as i32
        }
        _ => 0,
    };

    let DamageResult {
        raw_damage,
        is_crit,
        self_damage,
        lifesteal,
        bonus_turn,
    } = calculate_damage(board, player, enemy);

    let total_damage = raw_damage + venom_bonus;

    // Apply damage to enemy
    let damage_dealt = enemy.take_damage(total_damage);

    // Apply self-damage
    if self_damage > 0 {
        player.take_damage(self_damage);
    }

    // Apply weapon lifesteal
    if lifesteal > 0 {
        player.heal(lifesteal);
    }

    // Apply lifesteal potion: heal for 50% of damage dealt
    let potion_lifesteal = if lifesteal_potion_active {
        let heal = (damage_dealt as f64 * 0.5) as i32;
        player.heal(heal);
        heal
    } else {
        0
    };

    // Give XP to equipped books
    for book in &mut player.books {
        book.add_xp(1);
    }

    // Crystal scythe bonus: higher crystal spawn rate
    let crystal_bonus = match player.weapon.kind {
        super::equipment::WeaponKind::CrystalScythe => 0.04 * player.weapon.level as f64,
        _ => 0.0,
    };

    // Consume used tiles and generate new ones
    board.consume_selected(rng, crystal_bonus);

    // Purity book: chance to cleanse a bad tile
    let mut purity_cleansed = false;
    for book in &player.books {
        if matches!(book.kind, BookKind::Purity) {
            let chance = 0.10 * book.level as f64;
            if rng.gen_bool(chance.min(0.40)) {
                purity_cleansed = board.cleanse_one_random(rng);
            }
            break;
        }
    }

    // Health book: heal a small amount per word
    for book in &player.books {
        if matches!(book.kind, BookKind::Health) {
            let heal = book.level as i32;
            player.heal(heal);
            break;
        }
    }

    CombatResult {
        damage_dealt,
        is_crit,
        self_damage,
        lifesteal,
        potion_lifesteal,
        word,
        enemy_defeated: !enemy.is_alive(),
        bonus_turn,
        purity_cleansed,
        venom_bonus,
    }
}

pub struct EnemyTurnResult {
    pub damage_dealt: i32,
    pub dodged: bool,
    pub abilities_used: Vec<String>,
    pub healed: i32,
}

/// Resolve the enemy's turn
pub fn resolve_enemy_turn(
    board: &mut Board,
    player: &mut Player,
    enemy: &mut Enemy,
    rng: &mut impl Rng,
) -> EnemyTurnResult {
    let dodge_chance = player.dodge_chance + player.weapon.dodge_bonus();
    let dodged: bool = rng.gen_bool(dodge_chance.clamp(0.0, 1.0));

    let damage_dealt = if dodged {
        0
    } else {
        player.take_damage(enemy.attack)
    };

    let mut abilities_used = Vec::new();
    let mut healed = 0;

    let abilities = enemy.abilities.clone();
    for ability in &abilities {
        match ability {
            EnemyAbility::ApplyEffect {
                effect,
                count,
                timer,
            } => {
                board.apply_effect_to_random(rng, *effect, *count, *timer);
                abilities_used.push(format!(
                    "Applied {:?} to {} tiles!",
                    effect, count
                ));
            }
            EnemyAbility::HealSelf { amount } => {
                enemy.heal(*amount);
                healed += amount;
                abilities_used.push(format!("Healed for {} HP!", amount));
            }
            EnemyAbility::Enrage { bonus_attack } => {
                enemy.attack += bonus_attack;
                abilities_used.push(format!("Enraged! +{} attack!", bonus_attack));
            }
        }
    }

    // Tick board effects
    board.tick_effects(rng);

    EnemyTurnResult {
        damage_dealt,
        dodged,
        abilities_used,
        healed,
    }
}
