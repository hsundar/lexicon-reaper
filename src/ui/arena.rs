use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::game::state::CombatPhase;
use crate::generation::stage::Biome;

use super::icons;
use super::theme::{theme_for_biome, BiomeTheme};

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
            Constraint::Min(8),   // Characters + background
            Constraint::Length(1), // Ground line with props
            Constraint::Length(1), // Names
            Constraint::Length(1), // HP bars
            Constraint::Length(1), // Stats / constraint
        ])
        .split(inner);

    render_enemy_path(frame, chunks[0], combat);
    render_scene(frame, chunks[1], app, combat, &theme);
    render_ground_line(frame, chunks[2], app.biome, &theme);
    render_names(frame, chunks[3], combat);
    render_hp_bars(frame, chunks[4], app, combat);
    render_bottom_stats(frame, chunks[5], app, combat);
}

// ─── Enemy Path ───

fn render_enemy_path(frame: &mut Frame, area: Rect, combat: &crate::game::state::CombatState) {
    let total = combat.enemies.len();
    if total <= 1 {
        let line = Line::from(vec![Span::styled(
            format!("  [{}]", icons::DIAMOND),
            Style::default()
                .fg(Color::Rgb(255, 220, 80))
                .add_modifier(Modifier::BOLD),
        )]);
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
            Style::default().fg(color).add_modifier(if is_current {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        ));
        if i < total - 1 {
            let cc = if is_defeated {
                Color::Rgb(60, 120, 60)
            } else {
                Color::Rgb(60, 60, 80)
            };
            spans.push(Span::styled("───", Style::default().fg(cc)));
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ─── Scene: Background + Characters + Animation ───

fn render_scene(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
    theme: &BiomeTheme,
) {
    // 1) Render biome background across the full area
    render_background(frame, area, app.biome, theme);

    // 2) Split into player | gap | enemy and overlay characters
    let char_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Player character
    let player_art = get_player_pose(&combat.phase);
    let player_color = match &combat.phase {
        CombatPhase::EnemyHitAnim { dodged: true, .. } => Color::Rgb(100, 255, 100),
        CombatPhase::EnemyHitAnim { dodged: false, .. } => Color::Rgb(255, 100, 100),
        CombatPhase::PlayerAttackAnim { .. } => Color::Rgb(255, 220, 100),
        _ => Color::Rgb(150, 200, 255),
    };
    frame.render_widget(
        Paragraph::new(player_art)
            .style(Style::default().fg(player_color))
            .alignment(Alignment::Center),
        char_chunks[0],
    );

    // Animation gap
    render_animation_gap(frame, char_chunks[1], combat);

    // Enemy character
    let enemy = combat.current_enemy();
    let enemy_color = if enemy.is_boss {
        Color::Rgb(255, 80, 80)
    } else {
        theme.enemy_art
    };
    let enemy_display_color = match &combat.phase {
        CombatPhase::PlayerAttackAnim { ticks_remaining, .. } if *ticks_remaining < 5 => {
            Color::Rgb(255, 255, 200)
        }
        CombatPhase::EnemyDefeated { .. } => Color::Rgb(100, 100, 100),
        _ => enemy_color,
    };
    let enemy_art = get_enemy_art(&enemy.sprite_key);
    frame.render_widget(
        Paragraph::new(enemy_art)
            .style(Style::default().fg(enemy_display_color))
            .alignment(Alignment::Center),
        char_chunks[2],
    );
}

fn render_background(frame: &mut Frame, area: Rect, biome: Biome, theme: &BiomeTheme) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let bg_lines = get_biome_background(biome, w, h);
    let sky_style = Style::default().fg(theme.sky);

    for (i, line_str) in bg_lines.iter().enumerate() {
        if i >= h {
            break;
        }
        let y = area.y + i as u16;
        let rect = Rect::new(area.x, y, area.width, 1);

        // Color elements differently
        let colored_line = colorize_bg_line(line_str, biome, theme);
        frame.render_widget(Paragraph::new(Line::from(colored_line)), rect);
    }
}

fn get_biome_background(biome: Biome, w: usize, h: usize) -> Vec<String> {
    // All background uses only ASCII characters for reliable alignment
    let mut lines = Vec::with_capacity(h);

    match biome {
        Biome::Forest => {
            for row in 0..h {
                let line = match row {
                    0 => format_bg_line(w, &[" ,  ", "   .", "  , ", "    ", "  . ", " ,  ", "    "]),
                    1 => format_bg_line(w, &[" /\\  ", "    ", "   ", " /\\ ", "    ", "    ", "   "]),
                    2 => format_bg_line(w, &["/||\\ ", "   ", "   ", "/||\\ ", "   ", "    ", "  "]),
                    3 => format_bg_line(w, &[" || ", "   ", " * ", " || ", "   ", " ,, ", "  "]),
                    4 => format_bg_line(w, &[" || ", "   ", "   ", " || ", "   ", "    ", "  "]),
                    5 => format_bg_line(w, &[" || ", "   ", "   ", " || ", "   ", "    ", "  "]),
                    6 => format_bg_line(w, &[" || ", " . ", " . ", " || ", " . ", " .  ", "  "]),
                    _ => format_bg_line(w, &[" .  ", "    ", " .  ", "    ", " .  ", "    ", " . "]),
                };
                lines.push(line);
            }
        }
        Biome::Crypt => {
            for row in 0..h {
                let line = match row {
                    0 => format_bg_line(w, &["####", "    ", " i  ", "####", "    ", " i  ", "###"]),
                    1 => format_bg_line(w, &["|  |", "  \\/", "    ", "|  |", " \\/ ", "    ", "|  "]),
                    2 => format_bg_line(w, &["|  |", "    ", "    ", "|  |", "    ", "    ", "|  "]),
                    3 => format_bg_line(w, &["|  |", "    ", " *  ", "|  |", "    ", "    ", "|  "]),
                    4 => format_bg_line(w, &["|  |", "    ", "    ", "|  |", "    ", " d  ", "|  "]),
                    5 => format_bg_line(w, &["|  |", " -+-", "    ", "|  |", "  -+", "    ", "|  "]),
                    6 => format_bg_line(w, &["|  |", " -+-", "    ", "|  |", "  -+", "    ", "|  "]),
                    _ => format_bg_line(w, &["####", "    ", "  . ", "####", "    ", " .  ", "###"]),
                };
                lines.push(line);
            }
        }
        Biome::Volcano => {
            for row in 0..h {
                let line = match row {
                    0 => format_bg_line(w, &["  .*", "  . ", " *  ", "  . ", " .* ", "    ", " *  "]),
                    1 => format_bg_line(w, &["    ", " *  ", "    ", "  * ", "    ", " *  ", "    "]),
                    2 => format_bg_line(w, &["    ", "    ", " ^^ ", "    ", "    ", "    ", " ^^ "]),
                    3 => format_bg_line(w, &["  /\\ ", "    ", "    ", " /\\/\\", "    ", "    ", "    "]),
                    4 => format_bg_line(w, &[" /\\/\\", "    ", " *  ", "/\\/\\/", "    ", "    ", " /\\ "]),
                    5 => format_bg_line(w, &["/\\/\\/", " ~~ ", "    ", "/\\/\\/", "  ~~", "    ", "/\\/\\/"]),
                    6 => format_bg_line(w, &["~~~~", "~~~~", "    ", "~~~~", "~~~~", "    ", "~~~~"]),
                    _ => format_bg_line(w, &[" ~  ", "  ~ ", "    ", " ~  ", "  ~ ", "    ", " ~  "]),
                };
                lines.push(line);
            }
        }
        Biome::Abyss => {
            for row in 0..h {
                let line = match row {
                    0 => format_bg_line(w, &["   *", "  . ", "    ", " *  ", "   .", "    ", "  * "]),
                    1 => format_bg_line(w, &[" .  ", "    ", " *  ", "    ", " .  ", "  * ", "    "]),
                    2 => format_bg_line(w, &["    ", " <>  ", "    ", "    ", "  <> ", "    ", "    "]),
                    3 => format_bg_line(w, &["  : ", "    ", "  * ", "  : ", "    ", "    ", " :  "]),
                    4 => format_bg_line(w, &["  | ", "    ", "    ", "  | ", "    ", " <> ", "  | "]),
                    5 => format_bg_line(w, &["  | ", " .  ", "    ", "  | ", "  . ", "    ", "  | "]),
                    6 => format_bg_line(w, &["  : ", "    ", "  . ", "  : ", "    ", " .  ", "  : "]),
                    _ => format_bg_line(w, &["  . ", " .  ", "    ", "  . ", " .  ", "    ", "  . "]),
                };
                lines.push(line);
            }
        }
        Biome::Void => {
            for row in 0..h {
                let line = match row {
                    0 => format_bg_line(w, &["  + ", "    ", "  . ", "    ", " +  ", "    ", "  . "]),
                    1 => format_bg_line(w, &["    ", " .  ", "    ", " +  ", "    ", " .  ", "    "]),
                    2 => format_bg_line(w, &[" :: ", "    ", "    ", "  ::", "    ", "    ", " :: "]),
                    3 => format_bg_line(w, &["    ", " @  ", "    ", "    ", "  @ ", "    ", "    "]),
                    4 => format_bg_line(w, &[" .  ", "    ", " :: ", "  . ", "    ", " :: ", "    "]),
                    5 => format_bg_line(w, &["    ", "  . ", "    ", " .  ", "    ", "  . ", "    "]),
                    6 => format_bg_line(w, &["  ::", "    ", " .  ", "  ::", "    ", " .  ", "  ::"]),
                    _ => format_bg_line(w, &[" .  ", "    ", "    ", " .  ", "    ", "    ", " .  "]),
                };
                lines.push(line);
            }
        }
    }

    while lines.len() < h {
        lines.push(" ".repeat(w));
    }
    lines
}

fn format_bg_line(width: usize, segments: &[&str]) -> String {
    let mut line = String::new();
    let mut idx = 0;
    while line.len() < width {
        line.push_str(segments[idx % segments.len()]);
        idx += 1;
    }
    line.truncate(width);
    line
}

fn colorize_bg_line<'a>(line: &'a str, biome: Biome, theme: &BiomeTheme) -> Vec<Span<'a>> {
    // Render the entire line dimly in the biome's sky color
    // Special chars get slightly brighter prop colors
    let dim_style = Style::default().fg(theme.sky);
    vec![Span::styled(line, dim_style)]
}

// ─── Ground Line ───

fn render_ground_line(frame: &mut Frame, area: Rect, biome: Biome, theme: &BiomeTheme) {
    let w = area.width as usize;
    let pattern = match biome {
        Biome::Forest => "==,==.==w==.==*==.==,==.==w==.=",
        Biome::Crypt => "##i##-##d##-##*##-##i##-##d##-#",
        Biome::Volcano => "~~^^/\\~~/\\~~*~~~/\\^^~~^^/\\~~^^~",
        Biome::Abyss => "--<>--.--:--.-*--.--<>--.--:--.",
        Biome::Void => "::+::.::@::.::*::.::+::.::@::.",
    };

    let chars: Vec<char> = pattern.chars().collect();
    let mut spans: Vec<Span> = Vec::new();
    for i in 0..w {
        let ch = chars[i % chars.len()];
        let color = match ch {
            '*' => Color::Rgb(80, 200, 255),
            'w' | ',' => theme.prop2,
            '^' | '~' => theme.prop1,
            'i' | 'd' => theme.prop1,
            ':' | '<' | '>' | '@' | '+' => theme.prop2,
            '#' | '=' => theme.ground,
            '-' | '/' | '\\' => theme.ground,
            _ => theme.ground,
        };
        spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ─── Animation Gap ───

fn render_animation_gap(frame: &mut Frame, area: Rect, combat: &crate::game::state::CombatState) {
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
            let mut chars: Vec<char> = " ".repeat(total_width).chars().collect();
            let slash = "───⚔►";
            let slash_len = 5;
            let start = pos.saturating_sub(slash_len);
            for (i, ch) in slash.chars().enumerate() {
                let idx = start + i;
                if idx < total_width {
                    chars[idx] = ch;
                }
            }
            let line_chars: String = chars.into_iter().collect();
            let color = if *is_crit {
                Color::Rgb(255, 255, 80)
            } else {
                Color::Rgb(255, 180, 80)
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(line_chars, Style::default().fg(color)))),
                gap_rect,
            );
            // Floating damage
            if *ticks_remaining < 6 {
                let dmg_y = mid_y.saturating_sub(1 + (6 - *ticks_remaining) as u16 / 2);
                let dmg_x = area.x + area.width * 3 / 4;
                if dmg_y >= area.y && dmg_x + 8 <= area.x + area.width {
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
                            Style::default().fg(dmg_color).add_modifier(Modifier::BOLD),
                        ))),
                        dmg_rect,
                    );
                }
            }
        }
        CombatPhase::EnemyTurn { ticks_remaining }
        | CombatPhase::EnemyHitAnim { ticks_remaining, .. } => {
            let progress = 8u8.saturating_sub(*ticks_remaining);
            let total_width = area.width as usize;
            let pos = total_width.saturating_sub(progress as usize * total_width / 8);
            let mut chars: Vec<char> = " ".repeat(total_width).chars().collect();
            let slash = "◄⚔───";
            for (i, ch) in slash.chars().enumerate() {
                let idx = pos + i;
                if idx < total_width {
                    chars[idx] = ch;
                }
            }
            let line_chars: String = chars.into_iter().collect();
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    line_chars,
                    Style::default().fg(Color::Rgb(255, 80, 80)),
                ))),
                gap_rect,
            );
        }
        CombatPhase::Walking { ticks_remaining } => {
            let dots = match ticks_remaining % 4 {
                0 => "  >>>  ",
                1 => "   >>> ",
                2 => "    >>>",
                _ => " >>>   ",
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
            let text = if *ticks_remaining > 8 { "DEFEATED!" } else { "" };
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

// ─── Names, HP, Stats ───

fn render_names(frame: &mut Frame, area: Rect, combat: &crate::game::state::CombatState) {
    let enemy = combat.current_enemy();
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Grimm the Reaper",
            Style::default()
                .fg(Color::Rgb(150, 200, 255))
                .add_modifier(Modifier::BOLD),
        ))),
        halves[0],
    );

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

fn render_hp_bars(frame: &mut Frame, area: Rect, app: &App, combat: &crate::game::state::CombatState) {
    let enemy = combat.current_enemy();
    let player = &app.player;
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Length(4),
            Constraint::Percentage(48),
        ])
        .split(area);

    let p_ratio = player.hp as f64 / player.max_hp as f64;
    let p_color = hp_color(p_ratio);
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(p_color).bg(Color::Rgb(40, 40, 50)))
            .ratio(p_ratio.clamp(0.0, 1.0))
            .label(format!("{} {}/{}", icons::HEART, player.hp, player.max_hp)),
        halves[0],
    );

    let e_ratio = enemy.hp as f64 / enemy.max_hp as f64;
    let e_color = hp_color(e_ratio);
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(e_color).bg(Color::Rgb(40, 40, 50)))
            .ratio(e_ratio.clamp(0.0, 1.0))
            .label(format!("{} {}/{}", icons::HEART, enemy.hp, enemy.max_hp)),
        halves[2],
    );
}

fn render_bottom_stats(frame: &mut Frame, area: Rect, app: &App, combat: &crate::game::state::CombatState) {
    let enemy = combat.current_enemy();
    let player = &app.player;
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let stats = Line::from(vec![
        Span::styled(format!("  {} ", icons::SWORD), Style::default().fg(Color::Rgb(255, 120, 80))),
        Span::styled(format!("{}", player.base_attack), Style::default().fg(Color::White)),
        Span::styled(format!(" {} ", icons::SHIELD), Style::default().fg(Color::Rgb(120, 160, 255))),
        Span::styled(format!("{}", player.armor), Style::default().fg(Color::White)),
        Span::styled(format!(" {} ", icons::BOLT), Style::default().fg(Color::Rgb(255, 220, 80))),
        Span::styled(format!("{}%", (player.crit_chance * 100.0) as i32), Style::default().fg(Color::White)),
        Span::styled(format!(" {} ", icons::DIAMOND), Style::default().fg(Color::Rgb(255, 215, 0))),
        Span::styled(format!("{}", player.gems), Style::default().fg(Color::Rgb(255, 230, 100))),
    ]);
    frame.render_widget(Paragraph::new(stats), halves[0]);

    if let Some(constraint) = &enemy.word_constraint {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icons::WARNING), Style::default().fg(Color::Rgb(255, 200, 50))),
                Span::styled(
                    constraint.description(),
                    Style::default().fg(Color::Rgb(255, 200, 50)).add_modifier(Modifier::BOLD),
                ),
            ]))
            .alignment(Alignment::Right),
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

// ─── Player Character Art (8 lines) ───

fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_player_pose(phase: &CombatPhase) -> String {
    // All art uses only ASCII + basic box-drawing (single-width chars)
    // Each line is padded to exactly 14 chars for consistent alignment
    let art = match phase {
        CombatPhase::PlayerAttackAnim { ticks_remaining, .. } if *ticks_remaining > 5 => {
            concat!(
                "    .---.     \n",
                "    |o o|     \n",
                "    '---'     \n",
                "  .--+--.--.  \n",
                "  | /\\  |  |-+\n",
                "  |/  \\ |  | |\n",
                "  '-/--\\-'  | \n",
                "   /    \\   + \n",
            )
        }
        CombatPhase::EnemyHitAnim { dodged: true, .. } => {
            concat!(
                "      .---.   \n",
                "      |o o|   \n",
                "      '---'   \n",
                "    .--+--.   \n",
                "    | /\\  |+  \n",
                "     \\/  \\/ | \n",
                "      \\--/  | \n",
                "       \\/   + \n",
            )
        }
        CombatPhase::EnemyHitAnim { dodged: false, .. } => {
            concat!(
                "  .---.       \n",
                "  |x x|       \n",
                "  '---'       \n",
                " .--+--.      \n",
                " | ><>< | +   \n",
                " |  \\/  | |   \n",
                " '--/\\--' |   \n",
                "   /  \\   +   \n",
            )
        }
        CombatPhase::Walking { ticks_remaining } => {
            if ticks_remaining % 4 < 2 {
                concat!(
                    "    .---.     \n",
                    "    |o o|     \n",
                    "    '---'     \n",
                    "  .--+--.  +  \n",
                    "  | /\\  |  |  \n",
                    "  |/  \\ |  |  \n",
                    "  '-/--\\-' +  \n",
                    "   /   \\      \n",
                )
            } else {
                concat!(
                    "    .---.     \n",
                    "    |o o|     \n",
                    "    '---'     \n",
                    "  .--+--.  +  \n",
                    "  | /\\  |  |  \n",
                    "  |/  \\ |  |  \n",
                    "  '-/--\\-' +  \n",
                    "    \\  /      \n",
                )
            }
        }
        _ => {
            // Idle - reaper with scythe
            concat!(
                "    .---.     \n",
                "    |o o|     \n",
                "    '---'     \n",
                "  .--+--.     \n",
                "  | /\\  |  +  \n",
                "  |/  \\ |  |  \n",
                "  '-/--\\-' |  \n",
                "   /    \\  +  \n",
            )
        }
    };
    art.trim_end().to_string()
}

// ─── Enemy Art (8 lines) ───

fn get_enemy_art(sprite_key: &str) -> String {
    // All art uses only ASCII + basic box-drawing
    // Each line padded to consistent width per sprite
    let art = match sprite_key {
        "goblin" => concat!(
            "    /\\\n",
            "   /oo\\\n",
            "   \\></ \n",
            "    \\/  \n",
            "  .----.  \n",
            "  | /\\ |  \n",
            "  '-/\\-'  \n",
            "   /  \\   \n",
        ),
        "skeleton" => concat!(
            "   .==.   \n",
            "   |oo|   \n",
            "   '--'   \n",
            "  .=||=.  \n",
            "  |=||=|  \n",
            "  | || |== \n",
            "  '=/\\='  \n",
            "   /  \\   \n",
        ),
        "wolf" => concat!(
            "  /\\    /\\  \n",
            " /  \\--/  \\ \n",
            " | o    o | \n",
            " \\  <-->  / \n",
            "  '------'  \n",
            "   |/\\/\\|   \n",
            "   | |  ||  \n",
            "   '-'  ''  \n",
        ),
        "slime" => concat!(
            "  .-------. \n",
            " / o     o \\\n",
            " |  .---.  |\n",
            " |  '---'  |\n",
            " |  *   o  |\n",
            " \\  .---.  /\n",
            "  '-------' \n",
            "  ~~~~~~~~~~ \n",
        ),
        "boss_goblin" => concat!(
            "  W  /\\  W  \n",
            "    /oooo\\  \n",
            "    \\>>>>/  \n",
            " .--'----'-.\n",
            " | /\\/\\/\\ | \n",
            " |/  \\/  \\| \n",
            " '-/------\\'\n",
            "  //      \\\\\n",
        ),
        "boss_skeleton" => concat!(
            "  W .====. W\n",
            "    |oooo|  \n",
            "    '----'  \n",
            " .==+====+==.\n",
            " |==|====|==|\n",
            " |  | || |  |==\n",
            " '==/\\/\\==' \n",
            "  //      \\\\\n",
        ),
        "boss_wolf" => concat!(
            " W/\\  W  /\\W\n",
            " /  \\--/  \\ \n",
            " |oo    oo| \n",
            " \\  <===>  /\n",
            " .--------. \n",
            " | /\\/\\/\\ | \n",
            " |//      \\|\n",
            " '/        '\n",
        ),
        "boss_slime" => concat!(
            " W.---------.W\n",
            " / oo    oo   \\\n",
            " |  .------.  |\n",
            " |  '------'  |\n",
            " | ** oo ** oo|\n",
            " |  .------.  |\n",
            " '------------'\n",
            " ~~~~~~~~~~~~~~\n",
        ),
        _ => concat!(
            "  .-----. \n",
            "  | ??? | \n",
            "  | ??? | \n",
            "  '-----' \n",
            "  | ??? | \n",
            "  | ??? | \n",
            "  '-----' \n",
            "    ???    \n",
        ),
    };
    art.trim_end().to_string()
}
