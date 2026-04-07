use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::game::state::CombatPhase;

use super::icons;
use super::theme::theme_for_biome;

/// Render the unified battle arena showing player and enemy facing each other
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(format!(
            " {} Battle Arena ─── Stage {} │ {} ",
            icons::SKULL,
            app.stage_number,
            app.biome.name()
        ))
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 150, 100))
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let combat = match &app.combat {
        Some(c) => c,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Enemy path
            Constraint::Min(6),   // Characters + animation
            Constraint::Length(1), // Names
            Constraint::Length(1), // HP bars
            Constraint::Length(1), // Stats / constraint
        ])
        .split(inner);

    // --- Enemy path progression ---
    render_enemy_path(frame, chunks[0], combat);

    // --- Characters facing each other with animation ---
    render_characters(frame, chunks[1], app, combat);

    // --- Names ---
    render_names(frame, chunks[2], app, combat);

    // --- HP bars side by side ---
    render_hp_bars(frame, chunks[3], app, combat);

    // --- Player stats (left) + enemy constraint (right) ---
    render_bottom_stats(frame, chunks[4], app, combat);
}

fn render_enemy_path(
    frame: &mut Frame,
    area: Rect,
    combat: &crate::game::state::CombatState,
) {
    let total = combat.enemies.len();
    if total <= 1 {
        // Single enemy - just show current marker
        let line = Line::from(vec![
            Span::styled(
                format!("  [{}]", icons::DIAMOND),
                Style::default()
                    .fg(Color::Rgb(255, 220, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let current = combat.current_enemy_idx;
    let mut spans: Vec<Span> = vec![Span::raw("  ")];

    for i in 0..total {
        let is_current = i == current;
        let is_defeated = i < current;
        let is_walking = matches!(combat.phase, CombatPhase::Walking { .. }) && i == current + 1;

        let (node, color) = if is_current {
            (format!("[{}]", icons::DIAMOND), Color::Rgb(255, 220, 80))
        } else if is_defeated {
            (format!("[{}]", icons::CHECK), Color::Rgb(80, 180, 80))
        } else if is_walking {
            (format!("[{}]", icons::CHEVRON_RIGHT), Color::Rgb(255, 200, 80))
        } else {
            ("[ ]".to_string(), Color::Rgb(100, 100, 120))
        };

        spans.push(Span::styled(
            node,
            Style::default()
                .fg(color)
                .add_modifier(if is_current {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ));

        if i < total - 1 {
            let conn_color = if is_defeated {
                Color::Rgb(60, 120, 60)
            } else {
                Color::Rgb(60, 60, 80)
            };
            spans.push(Span::styled("───", Style::default().fg(conn_color)));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_characters(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
) {
    let enemy = combat.current_enemy();
    let theme = theme_for_biome(app.biome);

    // Split area into: player (left) | gap (center) | enemy (right)
    let char_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Player
            Constraint::Percentage(40), // Gap / animation
            Constraint::Percentage(30), // Enemy
        ])
        .split(area);

    // --- Player character ---
    let player_art = get_player_pose(&combat.phase);
    let player_color = match &combat.phase {
        CombatPhase::EnemyHitAnim { dodged: true, .. } => Color::Rgb(100, 255, 100),
        CombatPhase::EnemyHitAnim { dodged: false, .. } => Color::Rgb(255, 100, 100),
        CombatPhase::PlayerAttackAnim { .. } => Color::Rgb(255, 220, 100),
        _ => Color::Rgb(150, 200, 255),
    };
    let player_paragraph =
        Paragraph::new(player_art).style(Style::default().fg(player_color)).alignment(Alignment::Center);
    frame.render_widget(player_paragraph, char_chunks[0]);

    // --- Animation gap ---
    render_animation_gap(frame, char_chunks[1], combat);

    // --- Enemy character ---
    let enemy_art = get_enemy_art(&enemy.sprite_key);
    let enemy_color = if enemy.is_boss {
        Color::Rgb(255, 80, 80)
    } else {
        theme.enemy_art
    };

    // Flash enemy during hit
    let enemy_display_color = match &combat.phase {
        CombatPhase::PlayerAttackAnim {
            ticks_remaining, ..
        } if *ticks_remaining < 5 => Color::Rgb(255, 255, 200), // Flash white when hit
        CombatPhase::EnemyDefeated { .. } => Color::Rgb(100, 100, 100), // Gray when dead
        _ => enemy_color,
    };

    let enemy_paragraph = Paragraph::new(enemy_art)
        .style(Style::default().fg(enemy_display_color))
        .alignment(Alignment::Center);
    frame.render_widget(enemy_paragraph, char_chunks[2]);
}

fn render_animation_gap(
    frame: &mut Frame,
    area: Rect,
    combat: &crate::game::state::CombatState,
) {
    let mid_y = area.y + area.height / 2;
    let gap_rect = Rect::new(area.x, mid_y, area.width, 1);

    match &combat.phase {
        CombatPhase::PlayerAttackAnim {
            ticks_remaining,
            damage,
            is_crit,
            ..
        } => {
            let progress = 10u8.saturating_sub(*ticks_remaining);
            let total_width = area.width as usize;
            let pos = (progress as usize * total_width / 10).min(total_width.saturating_sub(1));

            let mut line_chars = " ".repeat(total_width);
            if pos < total_width {
                // Build slash trail
                let slash = "───⚔►";
                let slash_len = 5;
                let start = pos.saturating_sub(slash_len);
                let mut chars: Vec<char> = line_chars.chars().collect();
                for (i, ch) in slash.chars().enumerate() {
                    let idx = start + i;
                    if idx < total_width {
                        chars[idx] = ch;
                    }
                }
                line_chars = chars.into_iter().collect();
            }

            let color = if *is_crit {
                Color::Rgb(255, 255, 80)
            } else {
                Color::Rgb(255, 180, 80)
            };

            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    line_chars,
                    Style::default().fg(color),
                ))),
                gap_rect,
            );

            // Damage number floating above
            if *ticks_remaining < 6 {
                let dmg_y = mid_y.saturating_sub(1 + (6 - *ticks_remaining) as u16 / 2);
                let dmg_x = area.x + area.width * 3 / 4;
                if dmg_y >= area.y && dmg_x + 6 <= area.x + area.width {
                    let dmg_rect = Rect::new(dmg_x, dmg_y, 8, 1);
                    let dmg_text = if *is_crit {
                        format!("-{} !!!", damage)
                    } else {
                        format!("-{}", damage)
                    };
                    let dmg_color = if *is_crit {
                        Color::Rgb(255, 255, 80)
                    } else {
                        Color::Rgb(255, 120, 80)
                    };
                    frame.render_widget(
                        Paragraph::new(Line::from(Span::styled(
                            dmg_text,
                            Style::default()
                                .fg(dmg_color)
                                .add_modifier(Modifier::BOLD),
                        ))),
                        dmg_rect,
                    );
                }
            }
        }
        CombatPhase::EnemyTurn { ticks_remaining } | CombatPhase::EnemyHitAnim { ticks_remaining, .. } => {
            let progress = 8u8.saturating_sub(*ticks_remaining);
            let total_width = area.width as usize;
            let pos = total_width.saturating_sub(progress as usize * total_width / 8);

            let mut line_chars = " ".repeat(total_width);
            if pos < total_width {
                let slash = "◄⚔───";
                let mut chars: Vec<char> = line_chars.chars().collect();
                for (i, ch) in slash.chars().enumerate() {
                    let idx = pos + i;
                    if idx < total_width {
                        chars[idx] = ch;
                    }
                }
                line_chars = chars.into_iter().collect();
            }

            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    line_chars,
                    Style::default().fg(Color::Rgb(255, 80, 80)),
                ))),
                gap_rect,
            );
        }
        CombatPhase::Walking { ticks_remaining } => {
            // Walking dots animation
            let dots = match ticks_remaining % 4 {
                0 => "   >>>   ",
                1 => "    >>>  ",
                2 => "     >>> ",
                _ => "  >>>    ",
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^w$}", dots, w = area.width as usize),
                    Style::default().fg(Color::Rgb(150, 200, 255)),
                )))
                .alignment(Alignment::Center),
                gap_rect,
            );
        }
        CombatPhase::EnemyDefeated { ticks_remaining } => {
            let text = if *ticks_remaining > 8 {
                "   DEFEATED!   "
            } else {
                ""
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^w$}", text, w = area.width as usize),
                    Style::default()
                        .fg(Color::Rgb(255, 220, 80))
                        .add_modifier(Modifier::BOLD),
                )))
                .alignment(Alignment::Center),
                gap_rect,
            );
        }
        _ => {}
    }
}

fn render_names(
    frame: &mut Frame,
    area: Rect,
    _app: &App,
    combat: &crate::game::state::CombatState,
) {
    let enemy = combat.current_enemy();
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Player name
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Grimm",
            Style::default()
                .fg(Color::Rgb(150, 200, 255))
                .add_modifier(Modifier::BOLD),
        ))),
        halves[0],
    );

    // Enemy name
    let name_style = if enemy.is_boss {
        Style::default()
            .fg(Color::Rgb(255, 80, 80))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };
    let mut name_spans = vec![Span::styled(&enemy.name, name_style)];
    if enemy.is_boss {
        name_spans.push(Span::styled(
            format!(" {}BOSS", icons::STAR),
            Style::default()
                .fg(Color::Rgb(255, 200, 50))
                .add_modifier(Modifier::BOLD),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(name_spans)).alignment(Alignment::Right),
        halves[1],
    );
}

fn render_hp_bars(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
) {
    let enemy = combat.current_enemy();
    let player = &app.player;

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Length(4), // gap
            Constraint::Percentage(48),
        ])
        .split(area);

    // Player HP
    let p_ratio = player.hp as f64 / player.max_hp as f64;
    let p_color = hp_color(p_ratio);
    let p_gauge = Gauge::default()
        .gauge_style(Style::default().fg(p_color).bg(Color::Rgb(40, 40, 50)))
        .ratio(p_ratio.clamp(0.0, 1.0))
        .label(format!("{} {}/{}", icons::HEART, player.hp, player.max_hp));
    frame.render_widget(p_gauge, halves[0]);

    // Enemy HP
    let e_ratio = enemy.hp as f64 / enemy.max_hp as f64;
    let e_color = hp_color(e_ratio);
    let e_gauge = Gauge::default()
        .gauge_style(Style::default().fg(e_color).bg(Color::Rgb(40, 40, 50)))
        .ratio(e_ratio.clamp(0.0, 1.0))
        .label(format!("{} {}/{}", icons::HEART, enemy.hp, enemy.max_hp));
    frame.render_widget(e_gauge, halves[2]);
}

fn render_bottom_stats(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
) {
    let enemy = combat.current_enemy();
    let player = &app.player;

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Player compact stats
    let stats = Line::from(vec![
        Span::styled(format!("  {} ", icons::SWORD), Style::default().fg(Color::Rgb(255, 120, 80))),
        Span::styled(
            format!("{}", player.base_attack),
            Style::default().fg(Color::White),
        ),
        Span::styled(format!(" {} ", icons::SHIELD), Style::default().fg(Color::Rgb(120, 160, 255))),
        Span::styled(
            format!("{}", player.armor),
            Style::default().fg(Color::White),
        ),
        Span::styled(format!(" {} ", icons::BOLT), Style::default().fg(Color::Rgb(255, 220, 80))),
        Span::styled(
            format!("{}%", (player.crit_chance * 100.0) as i32),
            Style::default().fg(Color::White),
        ),
        Span::styled(format!(" {} ", icons::DIAMOND), Style::default().fg(Color::Rgb(255, 215, 0))),
        Span::styled(
            format!("{}", player.gems),
            Style::default().fg(Color::Rgb(255, 230, 100)),
        ),
    ]);
    frame.render_widget(Paragraph::new(stats), halves[0]);

    // Enemy constraint (right)
    if let Some(constraint) = &enemy.word_constraint {
        let line = Line::from(vec![
            Span::styled(
                format!("{} ", icons::WARNING),
                Style::default().fg(Color::Rgb(255, 200, 50)),
            ),
            Span::styled(
                constraint.description(),
                Style::default()
                    .fg(Color::Rgb(255, 200, 50))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Right),
            halves[1],
        );
    }
}

fn hp_color(ratio: f64) -> Color {
    if ratio > 0.6 {
        Color::Rgb(80, 220, 80)
    } else if ratio > 0.3 {
        Color::Rgb(255, 200, 50)
    } else {
        Color::Rgb(255, 60, 60)
    }
}

fn get_player_pose(phase: &CombatPhase) -> String {
    match phase {
        CombatPhase::PlayerAttackAnim { ticks_remaining, .. } if *ticks_remaining > 5 => {
            // Attack pose - scythe swung forward
            r#"    /\
   /||\──⌇
  / ||
 /  |
/  / \"#
                .to_string()
        }
        CombatPhase::EnemyHitAnim {
            dodged: true, ..
        } => {
            // Dodge pose - leaning back
            r#"      /\
     /||\
      ||  \
      || /
     /  \"#
                .to_string()
        }
        CombatPhase::EnemyHitAnim {
            dodged: false, ..
        } => {
            // Hit pose - recoiling
            r#"  /\
  /||\
  /|| \
   ||
  /  \"#
                .to_string()
        }
        CombatPhase::Walking { ticks_remaining } => {
            // Walk animation - alternate frames
            if ticks_remaining % 4 < 2 {
                r#"   /\
  /||\
  /||\
  / |
 /   \"#
                    .to_string()
            } else {
                r#"   /\
  /||\
  /||\
   | \
  /   \"#
                    .to_string()
            }
        }
        _ => {
            // Idle pose - standing with scythe
            r#"   /\
  /||\
  /||\⌇
  /  \
 /    \"#
                .to_string()
        }
    }
}

fn get_enemy_art(sprite_key: &str) -> String {
    match sprite_key {
        "goblin" => r#"  /\_/\
 ( o.o )
 (> ^ <)
  /| |\
 (_| |_)"#
            .to_string(),
        "skeleton" => r#"   _☠_
  /o.o\
  |=+=|
  /| |\
 (_/ \_)"#
            .to_string(),
        "wolf" => r#" /\    /\
/  \../  \
( ◉    ◉ )
 \  <>  /
  '----'"#
            .to_string(),
        "slime" => r#"  .-""-.
 / o  o \
|   __   |
 \ \__/ /
  '-..-'"#
            .to_string(),
        "boss_goblin" => r#" ♛___/\___
 / ◉    ◉ \
(  >>==<<  )
 \_|_/\_|_/
   |    |
  /|    |\"#
            .to_string(),
        "boss_skeleton" => r#" ♛  _☠☠_
   /◉  ◉\
  |==++=+|
   \_||_/
   /|  |\
  (_/  \_)"#
            .to_string(),
        "boss_wolf" => r#" ♛/\     /\
  /  \=*=/  \
 ( ◉◉   ◉◉ )
  \  <▽▽>  /
   '-=====-'"#
            .to_string(),
        "boss_slime" => r#" ♛.-""""-.
  / ◉   ◉ \
 |  /‾‾‾\  |
 |  \___/  |
  \________/"#
            .to_string(),
        _ => r#"  ??????
  ?    ?
  ?    ?
  ??????
  ? ?? ?"#
            .to_string(),
    }
}
