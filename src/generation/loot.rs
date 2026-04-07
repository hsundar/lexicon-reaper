use crate::game::equipment::BookKind;
use crate::game::player::Player;

/// Calculate gem reward for completing a stage
pub fn stage_gem_reward(stage: u32, is_boss: bool, player: &Player) -> u64 {
    let base = 10 + stage as u64 * 5;
    let boss_mult = if is_boss { 3.0 } else { 1.0 };

    // Wealth book multiplier
    let wealth_mult: f64 = player
        .books
        .iter()
        .filter(|b| matches!(b.kind, BookKind::Wealth))
        .map(|b| 1.0 + 0.15 * b.level as f64)
        .next()
        .unwrap_or(1.0);

    (base as f64 * boss_mult * wealth_mult) as u64
}
