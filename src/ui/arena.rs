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

// ─── Color palette for pixel sprites ───
// Each char in a sprite map references a color
const T: Color = Color::Reset; // Transparent

fn c(ch: char, biome_tint: bool) -> Color {
    match ch {
        '.' => T,                              // transparent
        'K' => Color::Rgb(30, 30, 40),         // black/dark
        'G' => Color::Rgb(80, 80, 100),        // gray
        'g' => Color::Rgb(120, 120, 140),      // light gray
        'W' => Color::Rgb(220, 220, 230),      // white
        'R' => Color::Rgb(200, 50, 50),        // red
        'r' => Color::Rgb(255, 100, 100),      // light red
        'B' => Color::Rgb(60, 80, 180),        // blue
        'b' => Color::Rgb(120, 160, 255),      // light blue
        'P' => Color::Rgb(140, 60, 180),       // purple
        'p' => Color::Rgb(200, 130, 255),      // light purple
        'Y' => Color::Rgb(200, 170, 40),       // yellow/gold
        'y' => Color::Rgb(255, 230, 100),      // bright yellow
        'O' => Color::Rgb(200, 120, 40),       // orange
        'o' => Color::Rgb(255, 180, 80),       // light orange
        'N' => Color::Rgb(60, 140, 60),        // green
        'n' => Color::Rgb(100, 220, 100),      // light green
        'C' => Color::Rgb(40, 120, 140),       // cyan/teal
        'c' => Color::Rgb(80, 200, 220),       // light cyan
        'S' => Color::Rgb(160, 120, 80),       // skin/tan
        's' => Color::Rgb(200, 160, 110),      // light skin
        'M' => Color::Rgb(100, 60, 40),        // brown
        'm' => Color::Rgb(160, 110, 70),       // light brown
        'E' => Color::Rgb(200, 200, 50),       // eye yellow
        'e' => Color::Rgb(255, 50, 50),        // eye red
        _ => T,
    }
}

/// A pixel sprite: rows of color-character codes, rendered using half-blocks.
/// Each row pair produces one terminal line using ▀ with fg=top, bg=bottom.
struct PixelSprite {
    rows: Vec<Vec<char>>,
    width: usize,
    height: usize, // in pixel rows (2 pixel rows = 1 terminal line)
}

impl PixelSprite {
    fn from_lines(lines: &[&str]) -> Self {
        let rows: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let height = rows.len();
        Self {
            rows,
            width,
            height,
        }
    }

    /// Render sprite into a frame area, centered horizontally
    fn render(&self, frame: &mut Frame, area: Rect, flash: Option<Color>) {
        let term_lines = (self.height + 1) / 2;
        let x_offset = area
            .width
            .saturating_sub(self.width as u16) / 2;

        for term_row in 0..term_lines {
            let y = area.y + term_row as u16;
            if y >= area.y + area.height {
                break;
            }

            let top_row = term_row * 2;
            let bot_row = term_row * 2 + 1;

            let mut spans: Vec<Span> = Vec::new();

            // Left padding
            if x_offset > 0 {
                spans.push(Span::raw(" ".repeat(x_offset as usize)));
            }

            for col in 0..self.width {
                let top_ch = self.rows.get(top_row).and_then(|r| r.get(col)).copied().unwrap_or('.');
                let bot_ch = self.rows.get(bot_row).and_then(|r| r.get(col)).copied().unwrap_or('.');

                let top_color = flash.unwrap_or_else(|| c(top_ch, false));
                let bot_color = flash.unwrap_or_else(|| c(bot_ch, false));

                let is_top_transparent = top_ch == '.';
                let is_bot_transparent = bot_ch == '.';

                if is_top_transparent && is_bot_transparent {
                    spans.push(Span::raw(" "));
                } else if is_top_transparent {
                    // Only bottom pixel: use lower half block with fg=bottom
                    spans.push(Span::styled(
                        "\u{2584}", // ▄
                        Style::default().fg(bot_color),
                    ));
                } else if is_bot_transparent {
                    // Only top pixel: use upper half block with fg=top
                    spans.push(Span::styled(
                        "\u{2580}", // ▀
                        Style::default().fg(top_color),
                    ));
                } else {
                    // Both pixels: upper half block with fg=top, bg=bottom
                    spans.push(Span::styled(
                        "\u{2580}", // ▀
                        Style::default().fg(top_color).bg(bot_color),
                    ));
                }
            }

            let line_rect = Rect::new(area.x, y, area.width, 1);
            frame.render_widget(Paragraph::new(Line::from(spans)), line_rect);
        }
    }
}

// ─── Main render ───

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(format!(
            " {} Battle Arena --- Stage {} | {} ",
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
            Constraint::Length(1), // Ground
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
            spans.push(Span::styled("---", Style::default().fg(cc)));
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ─── Scene ───

fn render_scene(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
    theme: &BiomeTheme,
) {
    // Background
    render_background(frame, area, app.biome, theme);

    let char_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Player sprite
    let player_sprite = get_player_sprite(&combat.phase);
    let player_flash = match &combat.phase {
        CombatPhase::EnemyHitAnim { dodged: false, ticks_remaining, .. } if *ticks_remaining > 3 => {
            Some(Color::Rgb(255, 80, 80))
        }
        _ => None,
    };
    player_sprite.render(frame, char_chunks[0], player_flash);

    // Animation gap
    render_animation_gap(frame, char_chunks[1], combat);

    // Enemy sprite
    let enemy = combat.current_enemy();
    let enemy_sprite = get_enemy_sprite(&enemy.sprite_key);
    let enemy_flash = match &combat.phase {
        CombatPhase::PlayerAttackAnim { ticks_remaining, .. } if *ticks_remaining < 5 => {
            Some(Color::Rgb(255, 255, 200))
        }
        CombatPhase::EnemyDefeated { .. } => Some(Color::Rgb(80, 80, 80)),
        _ => None,
    };
    enemy_sprite.render(frame, char_chunks[2], enemy_flash);
}

fn render_background(frame: &mut Frame, area: Rect, biome: Biome, theme: &BiomeTheme) {
    let w = area.width as usize;
    let h = area.height as usize;

    for row in 0..h {
        let y = area.y + row as u16;
        let pattern = get_bg_pattern(biome, row, w);
        let mut spans: Vec<Span> = Vec::new();
        for ch in pattern.chars().take(w) {
            let color = match ch {
                '*' | '+' => theme.prop1,
                ',' | '.' => theme.sky,
                '|' | '#' => theme.ground,
                '~' | '^' => theme.prop1,
                ':' | '@' => theme.prop2,
                _ => theme.sky,
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, y, area.width, 1),
        );
    }
}

fn get_bg_pattern(biome: Biome, row: usize, w: usize) -> String {
    let segs: &[&str] = match biome {
        Biome::Forest => match row % 8 {
            0 => &[" .  ", "   .", " .  ", "    ", "  . ", "    "],
            1 => &[" /\\ ", "    ", "    ", " /\\ ", "    ", "    "],
            2 => &["/||\\ ", "    ", "   ", "/||\\ ", "    ", "   "],
            3 => &[" ||  ", "  * ", "   ", " ||  ", "    ", " ,  "],
            4 => &[" ||  ", "    ", "   ", " ||  ", "    ", "    "],
            5 => &[" ||  ", "    ", "   ", " ||  ", "    ", "    "],
            6 => &[" ||  ", " .  ", " . ", " ||  ", " .  ", "    "],
            _ => &[" .   ", "     ", " .   ", "     ", " .   ", "    "],
        },
        Biome::Crypt => match row % 8 {
            0 => &["#### ", "     ", "     ", "#### ", "     ", "    "],
            1 => &["|  | ", "  +  ", "     ", "|  | ", "  +  ", "    "],
            2 => &["|  | ", "     ", "     ", "|  | ", "     ", "    "],
            3 => &["|  | ", "     ", "  *  ", "|  | ", "     ", "    "],
            4 => &["|  | ", "     ", "     ", "|  | ", "     ", "    "],
            5 => &["|  | ", "  .  ", "     ", "|  | ", "  .  ", "    "],
            6 => &["|  | ", "     ", "     ", "|  | ", "     ", "    "],
            _ => &["#### ", "  .  ", "     ", "#### ", "  .  ", "    "],
        },
        Biome::Volcano => match row % 8 {
            0 => &["  * ", "  . ", "    ", " *  ", "  . ", "    "],
            1 => &["    ", " *  ", "    ", "    ", " *  ", "    "],
            2 => &["    ", "    ", " ^^ ", "    ", "    ", " ^^ "],
            3 => &[" /\\  ", "    ", "    ", " /\\ ", "    ", "    "],
            4 => &["/\\/\\ ", "    ", "    ", "/\\/\\", "    ", "    "],
            5 => &["/\\/\\ ", " ~~ ", "    ", "/\\/\\", " ~~ ", "    "],
            6 => &["~~~~ ", "~~~~", "    ", "~~~~", "~~~~", "    "],
            _ => &[" ~   ", "  ~  ", "     ", " ~   ", "  ~  ", "    "],
        },
        Biome::Abyss => match row % 8 {
            0 => &["  * ", "  . ", "    ", " *  ", "  . ", "    "],
            1 => &[" .  ", "    ", " *  ", "    ", " .  ", "    "],
            2 => &["    ", " :  ", "    ", "    ", "  : ", "    "],
            3 => &["  | ", "    ", "  * ", "  | ", "    ", "    "],
            4 => &["  | ", "    ", "    ", "  | ", "    ", "    "],
            5 => &["  | ", " .  ", "    ", "  | ", "  . ", "    "],
            6 => &["  : ", "    ", "  . ", "  : ", "    ", "    "],
            _ => &["  .  ", " .   ", "     ", "  .  ", " .   ", "    "],
        },
        Biome::Void => match row % 8 {
            0 => &["  + ", "    ", "  . ", "    ", " +  ", "    "],
            1 => &["    ", " .  ", "    ", " +  ", "    ", "    "],
            2 => &[" :: ", "    ", "    ", " :: ", "    ", "    "],
            3 => &["    ", " @  ", "    ", "    ", "  @ ", "    "],
            4 => &[" .  ", "    ", " :: ", "    ", " .  ", "    "],
            5 => &["    ", "  . ", "    ", " .  ", "    ", "    "],
            6 => &[" :: ", "    ", " .  ", " :: ", "    ", "    "],
            _ => &[" .   ", "     ", "     ", " .   ", "     ", "    "],
        },
    };

    let mut line = String::new();
    let mut idx = 0;
    while line.len() < w {
        line.push_str(segs[idx % segs.len()]);
        idx += 1;
    }
    line.truncate(w);
    line
}

// ─── Ground Line ───

fn render_ground_line(frame: &mut Frame, area: Rect, biome: Biome, theme: &BiomeTheme) {
    let w = area.width as usize;
    let pattern: &str = match biome {
        Biome::Forest => "==,==.==w==.==*==.==,==.==w==.=",
        Biome::Crypt => "##i##-##d##-##*##-##i##-##d##-#",
        Biome::Volcano => "~~^^/\\~~/\\~~*~~~/\\^^~~^^/\\~~^^~",
        Biome::Abyss => "--::--.--:--.-*--.--::--.-:--.-",
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
            ':' | '@' | '+' => theme.prop2,
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
            let w = area.width as usize;
            let pos = (progress as usize * w / 10).min(w.saturating_sub(1));
            let mut chars: Vec<char> = vec![' '; w];
            let slash: Vec<char> = "---=>".chars().collect();
            for (i, &ch) in slash.iter().enumerate() {
                let idx = pos.saturating_sub(slash.len()) + i;
                if idx < w { chars[idx] = ch; }
            }
            let line_str: String = chars.into_iter().collect();
            let color = if *is_crit { Color::Rgb(255, 255, 80) } else { Color::Rgb(255, 180, 80) };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(line_str, Style::default().fg(color)))),
                gap_rect,
            );
            if *ticks_remaining < 6 {
                let dmg_y = mid_y.saturating_sub(1 + (6 - *ticks_remaining) as u16 / 2);
                let dmg_x = area.x + area.width * 3 / 4;
                if dmg_y >= area.y && dmg_x + 8 <= area.x + area.width {
                    let dmg_rect = Rect::new(dmg_x, dmg_y, 8, 1);
                    let dmg_text = if *is_crit { format!("-{} !!!", damage) } else { format!("-{}", damage) };
                    let dmg_color = if *is_crit { Color::Rgb(255, 255, 80) } else { Color::Rgb(255, 120, 80) };
                    frame.render_widget(
                        Paragraph::new(Line::from(Span::styled(dmg_text, Style::default().fg(dmg_color).add_modifier(Modifier::BOLD)))),
                        dmg_rect,
                    );
                }
            }
        }
        CombatPhase::EnemyTurn { ticks_remaining } | CombatPhase::EnemyHitAnim { ticks_remaining, .. } => {
            let progress = 8u8.saturating_sub(*ticks_remaining);
            let w = area.width as usize;
            let pos = w.saturating_sub(progress as usize * w / 8);
            let mut chars: Vec<char> = vec![' '; w];
            let slash: Vec<char> = "<=---".chars().collect();
            for (i, &ch) in slash.iter().enumerate() {
                let idx = pos + i;
                if idx < w { chars[idx] = ch; }
            }
            let line_str: String = chars.into_iter().collect();
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(line_str, Style::default().fg(Color::Rgb(255, 80, 80))))),
                gap_rect,
            );
        }
        CombatPhase::Walking { ticks_remaining } => {
            let w = area.width as usize;
            let dots = match ticks_remaining % 4 {
                0 => "  >>>  ",
                1 => "   >>> ",
                2 => "    >>>",
                _ => " >>>   ",
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^w$}", dots, w = w),
                    Style::default().fg(Color::Rgb(150, 200, 255)),
                ))).alignment(Alignment::Center),
                gap_rect,
            );
        }
        CombatPhase::EnemyDefeated { ticks_remaining } => {
            let text = if *ticks_remaining > 8 { "DEFEATED!" } else { "" };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^w$}", text, w = area.width as usize),
                    Style::default().fg(Color::Rgb(255, 220, 80)).add_modifier(Modifier::BOLD),
                ))).alignment(Alignment::Center),
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
            Style::default().fg(Color::Rgb(150, 200, 255)).add_modifier(Modifier::BOLD),
        ))),
        halves[0],
    );

    let name_style = if enemy.is_boss {
        Style::default().fg(Color::Rgb(255, 80, 80)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    };
    let mut name_spans = vec![Span::styled(&enemy.name, name_style)];
    if enemy.is_boss {
        name_spans.push(Span::styled(
            format!(" {}BOSS", icons::STAR),
            Style::default().fg(Color::Rgb(255, 200, 50)).add_modifier(Modifier::BOLD),
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
        .constraints([Constraint::Percentage(48), Constraint::Length(4), Constraint::Percentage(48)])
        .split(area);

    let p_ratio = player.hp as f64 / player.max_hp as f64;
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(hp_color(p_ratio)).bg(Color::Rgb(40, 40, 50)))
            .ratio(p_ratio.clamp(0.0, 1.0))
            .label(format!("{} {}/{}", icons::HEART, player.hp, player.max_hp)),
        halves[0],
    );

    let e_ratio = enemy.hp as f64 / enemy.max_hp as f64;
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(hp_color(e_ratio)).bg(Color::Rgb(40, 40, 50)))
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
                Span::styled(constraint.description(), Style::default().fg(Color::Rgb(255, 200, 50)).add_modifier(Modifier::BOLD)),
            ])).alignment(Alignment::Right),
            halves[1],
        );
    }
}

fn hp_color(ratio: f64) -> Color {
    if ratio > 0.6 { Color::Rgb(80, 220, 80) }
    else if ratio > 0.3 { Color::Rgb(255, 200, 50) }
    else { Color::Rgb(255, 60, 60) }
}

// ─── Pixel Sprites ───
// Each char = 1 pixel. Two rows = 1 terminal line via half-block rendering.
// Palette: K=dark, G=gray, g=light gray, W=white, R=red, r=light red
//          B=blue, b=light blue, P=purple, p=light purple, Y=gold, y=yellow
//          O=orange, N=green, n=light green, S=skin, M=brown, E=eye yellow
//          e=eye red, .=transparent

fn get_player_sprite(phase: &CombatPhase) -> PixelSprite {
    match phase {
        CombatPhase::PlayerAttackAnim { ticks_remaining, .. } if *ticks_remaining > 5 => {
            PixelSprite::from_lines(&[
                "....PPPP....",
                "...PPPPPP...",
                "..PP.EE.PP..",
                "..PPPPPPPP..",
                "...BBBBBB...",
                "..BBBBBBBB..",
                ".BBB.BB.BBBM",
                ".BB..BB..BBM",
                "..BB.BB.BB.M",
                "...BBBBBB.MM",
                "....B..B....",
                "...BB..BB...",
            ])
        }
        CombatPhase::EnemyHitAnim { dodged: false, .. } => {
            PixelSprite::from_lines(&[
                "..PPPP......",
                ".PPPPPP.....",
                "PP.ee.PP....",
                "PPPPPPPP....",
                ".BBBBBB.....",
                "BBBBBBBB....",
                "BBB.BB.BBB..",
                "BB..BB..BB..",
                ".BB.BB.BB...",
                "..BBBBBB....",
                "...B..B.....",
                "..BB..BB....",
            ])
        }
        CombatPhase::EnemyHitAnim { dodged: true, .. } => {
            PixelSprite::from_lines(&[
                "......PPPP..",
                ".....PPPPPP.",
                "....PP.EE.PP",
                "....PPPPPPPP",
                ".....BBBBBB.",
                "....BBBBBBBB",
                "...BBB.BB.BB",
                "...BB..BB..B",
                "....BB.BB.B.",
                ".....BBBBBB.",
                "......B..B..",
                ".....BB..BB.",
            ])
        }
        CombatPhase::Walking { ticks_remaining } => {
            if ticks_remaining % 4 < 2 {
                PixelSprite::from_lines(&[
                    "....PPPP....",
                    "...PPPPPP...",
                    "..PP.EE.PP..",
                    "..PPPPPPPP..",
                    "...BBBBBB...",
                    "..BBBBBBBBMM",
                    "..BBB.BB.BMM",
                    "..BB..BB..M.",
                    "...BB.BB.B..",
                    "....BBBBBB..",
                    "....B...B...",
                    "...BB....B..",
                ])
            } else {
                PixelSprite::from_lines(&[
                    "....PPPP....",
                    "...PPPPPP...",
                    "..PP.EE.PP..",
                    "..PPPPPPPP..",
                    "...BBBBBB...",
                    "..BBBBBBBBMM",
                    "..BBB.BB.BMM",
                    "..BB..BB..M.",
                    "...BB.BB.B..",
                    "....BBBBBB..",
                    ".....B..B...",
                    "....B..BB...",
                ])
            }
        }
        _ => {
            // Idle
            PixelSprite::from_lines(&[
                "....PPPP....",
                "...PPPPPP...",
                "..PP.EE.PP..",
                "..PPPPPPPP..",
                "...BBBBBB...",
                "..BBBBBBBB..",
                "..BBB.BB.BBM",
                "..BB..BB..BM",
                "...BB.BB.BMM",
                "....BBBBBBM.",
                "....B..B....",
                "...BB..BB...",
            ])
        }
    }
}

fn get_enemy_sprite(sprite_key: &str) -> PixelSprite {
    match sprite_key {
        "goblin" => PixelSprite::from_lines(&[
            "..NN....NN..",
            "..NNNNNNNN..",
            ".NNN.ee.NNN.",
            ".NNNNNNNNNN.",
            "..NNNNNNNN..",
            ".SSSSSSSSSS.",
            ".SS.SSSS.SS.",
            ".SS..SS..SS.",
            "..SS.SS.SS..",
            "...SSSSSS...",
            "...SS..SS...",
            "..SSS..SSS..",
        ]),
        "skeleton" => PixelSprite::from_lines(&[
            "...WWWWWW...",
            "..WWWWWWWW..",
            "..WW.ee.WW..",
            "..WWWWWWWW..",
            "...WW..WW...",
            "..WWWWWWWW..",
            "..WW.WW.WW..",
            "..WWWWWWWW..",
            "...WW..WW...",
            "...WW..WW...",
            "...WW..WW...",
            "..WWW..WWW..",
        ]),
        "wolf" => PixelSprite::from_lines(&[
            ".MM......MM.",
            "MMMM..MMMM.",
            "MM.MMMM.MM.",
            "MM.e..e.MM..",
            ".MMMMMMMM..",
            "..MMMMMM...",
            ".MMMMMMMM..",
            ".MM.MM.MM...",
            ".MM.MM.MM...",
            ".MM.MM.MM...",
            ".MM....MM...",
            "MMM....MMM..",
        ]),
        "slime" => PixelSprite::from_lines(&[
            "...nnnnnn...",
            "..nnnnnnnn..",
            ".nn.ee.nn.n.",
            ".nnnnnnnnnn.",
            ".nnnnnnnnnn.",
            ".nn.Yn.Cn.n.",
            ".nnnnnnnnnn.",
            "..nnnnnnnn..",
            "...nnnnnn...",
            "..n..nn..n..",
            ".n...nn...n.",
            "............",
        ]),
        "boss_goblin" => PixelSprite::from_lines(&[
            "..YY........YY..",
            "...NNNN..NNNN...",
            "...NNNNNNNNNN...",
            "..NNN.ee.ee.NNN.",
            "..NNNNNNNNNNNN..",
            "..SSSSSSSSSSSS..",
            ".SSS.SSSSSS.SSS.",
            ".SS...SSSS...SS.",
            "..SSS.SSSS.SSS..",
            "...SSSSSSSSSS...",
            "....SS....SS....",
            "...SSS....SSS...",
        ]),
        "boss_skeleton" => PixelSprite::from_lines(&[
            "..YY........YY..",
            "...WWWWWWWWWW...",
            "..WWWWWWWWWWWW..",
            "..WW..ee.ee.WW..",
            "..WWWWWWWWWWWW..",
            "...WWWW..WWWW...",
            "..WWWWWWWWWWWW..",
            "..WW..WWWW..WW..",
            "..WWWWWWWWWWWW..",
            "...WWWW..WWWW...",
            "...WWW....WWW...",
            "..WWWW....WWWW..",
        ]),
        "boss_wolf" => PixelSprite::from_lines(&[
            ".YMM......MMY...",
            "MMMMM..MMMMM....",
            "MM.MMMMM.MMMMM..",
            "MM.ee..ee.MMMM..",
            ".MMMMMMMMMMMM...",
            "..MMMMMMMMMM....",
            ".MMMMMMMMMMMM...",
            ".MMM.MMMM.MMM...",
            ".MM..MMMM..MM...",
            ".MM..MMMM..MM...",
            ".MM........MM...",
            "MMMM......MMMM..",
        ]),
        "boss_slime" => PixelSprite::from_lines(&[
            "..YY..nnnnnn..YY",
            "...nnnnnnnnnnn..",
            "..nnnnnnnnnnnn..",
            ".nn..ee..ee..nn.",
            ".nnnnnnnnnnnnnn.",
            ".nn.Yn.Cn.Yn.nn.",
            ".nnnnnnnnnnnnnn.",
            ".nnnnnnnnnnnnnn.",
            "..nnnnnnnnnnnn..",
            "...nnnnnnnnnn...",
            "..n..nn..nn..n..",
            ".n...nn..nn...n.",
        ]),
        _ => PixelSprite::from_lines(&[
            "..GGGGGG..",
            ".GGGGGGGG.",
            ".GG.GG.GG.",
            ".GGGGGGGG.",
            "..GGGGGG..",
            ".GGGGGGGG.",
            ".GG.GG.GG.",
            ".GGGGGGGG.",
            "..GGGGGG..",
            "...GG.GG..",
            "..GGG.GGG.",
            "............",
        ]),
    }
}
