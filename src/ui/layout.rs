use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::game::state::CombatPhase;

use super::arena;
use super::message_log;
use super::icons;
use super::theme::theme_for_biome;
use super::tile_grid;
use super::word_bar;

pub fn render_combat(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let theme = theme_for_biome(app.biome);

    // Main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar with border
            Constraint::Length(12), // Battle arena
            Constraint::Length(7),  // Word tray (full width)
            Constraint::Min(26),   // Bottom: log | grid | word bar
            Constraint::Length(1), // Keybinding bar
        ])
        .split(size);

    // Title bar
    render_title_bar(frame, chunks[0], app, &theme);

    // Battle arena
    arena::render(frame, chunks[1], app);

    // Word tray spanning full width
    tile_grid::render_tray(frame, chunks[2], app);

    // Bottom section: combat log (left) | letter grid (center) | word bar (right)
    let grid_width = 62u16;
    let word_width = 28u16;

    let bot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(28),             // Combat log (left, flexible)
            Constraint::Length(grid_width),   // Letter grid (center, fixed)
            Constraint::Length(word_width),   // Word bar (right, fixed)
        ])
        .split(chunks[3]);

    message_log::render(frame, bot_chunks[0], app);
    tile_grid::render_grid(frame, bot_chunks[1], app);
    word_bar::render(frame, bot_chunks[2], app);

    // Keybinding bar
    render_keybind_bar(frame, chunks[4], app);

    // Overlays
    render_phase_overlay(frame, size, app);

    if app.show_help {
        render_help_overlay(frame, size);
    }
    if app.show_stats {
        render_stats_overlay(frame, size, app);
    }
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App, theme: &super::theme::BiomeTheme) {
    let biome_name = app.biome.name();

    let turn_info = app
        .combat
        .as_ref()
        .map(|c| format!("Turn {}", c.turn_count))
        .unwrap_or_default();

    let title = Line::from(vec![
        Span::styled(
            format!(" {} LEXICON REAPER ", icons::SWORD),
            Style::default()
                .fg(Color::Rgb(255, 80, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Stage {} ", app.stage_number),
            Style::default()
                .fg(Color::Rgb(255, 220, 100))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} ", biome_name),
            Style::default().fg(theme.primary),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(turn_info, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default());

    let paragraph = Paragraph::new(title).block(block);
    frame.render_widget(paragraph, area);
}

fn render_keybind_bar(frame: &mut Frame, area: Rect, _app: &App) {
    let sep = Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 70)));
    let keybinds = Line::from(vec![
        Span::styled(" ←↑↓→", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Move", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("Space", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Select", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("Enter", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Submit", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("S", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Shuffle", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("P", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Potion", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("Esc", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Clear", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("Bksp", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Undo", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("Tab", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Stats", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("?", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Help", Style::default().fg(Color::Rgb(180, 180, 190))),
    ]);
    frame.render_widget(Paragraph::new(keybinds), area);
}

fn render_phase_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let combat = match &app.combat {
        Some(c) => c,
        None => return,
    };

    match &combat.phase {
        CombatPhase::StageVictory { gems_earned, .. } => {
            let popup_w = 36;
            let popup_h = 5;
            let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
            let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
            let popup_area = Rect::new(x, y, popup_w, popup_h);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 220, 80)))
                .title(format!(" {} VICTORY {} ", icons::STAR, icons::STAR))
                .title_style(
                    Style::default()
                        .fg(Color::Rgb(255, 220, 80))
                        .add_modifier(Modifier::BOLD),
                );

            let text = Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("Stage {} Complete!", app.stage_number),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("+{} gems", gems_earned),
                    Style::default()
                        .fg(Color::Rgb(255, 220, 80))
                        .add_modifier(Modifier::BOLD),
                )),
            ])
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

            // Clear background
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Rgb(20, 20, 30))),
                popup_area,
            );
            frame.render_widget(text, popup_area);
        }
        CombatPhase::Defeat => {
            let popup_w = 30;
            let popup_h = 3;
            let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
            let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
            let popup_area = Rect::new(x, y, popup_w, popup_h);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(200, 50, 50)));

            let text = Paragraph::new(Line::from(Span::styled(
                "YOU HAVE FALLEN...",
                Style::default()
                    .fg(Color::Rgb(255, 60, 60))
                    .add_modifier(Modifier::BOLD),
            )))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(text, popup_area);
        }
        _ => {}
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup_w = 50.min(area.width.saturating_sub(4));
    let popup_h = 18.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(x, y, popup_w, popup_h);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 180, 255)))
        .title(" Help ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(100, 180, 255))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let help_text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(" Controls:", Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  Arrow keys  Navigate tile grid", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Space       Select/deselect tile", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Backspace   Undo last selection", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Enter       Submit word", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Esc         Clear all selections", Style::default().fg(Color::White))),
        Line::from(Span::styled("  S           Shuffle board (costs turn)", Style::default().fg(Color::White))),
        Line::from(Span::styled("  P           Use potion", Style::default().fg(Color::White))),
        Line::from(Span::styled("  Tab         View run statistics", Style::default().fg(Color::White))),
        Line::from(Span::styled("  ?           Toggle this help", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(" Tips:", Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  Longer words deal more damage", Style::default().fg(Color::Rgb(160, 160, 175)))),
        Line::from(Span::styled("  Gold tiles are worth 3x points", Style::default().fg(Color::Rgb(160, 160, 175)))),
        Line::from(Span::styled("  Press any key to close", Style::default().fg(Color::Rgb(100, 100, 120)))),
    ])
    .block(block);

    frame.render_widget(help_text, popup_area);
}

fn render_stats_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let s = &app.stats;
    let history_lines = s.word_history.len().min(8);
    let popup_h = (18 + history_lines as u16).min(area.height.saturating_sub(4));
    let popup_w = 50.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(x, y, popup_w, popup_h);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(200, 180, 100)))
        .title(" Run Statistics ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 220, 80))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let stat_line = |label: &str, value: String| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {:<22}", label), Style::default().fg(Color::Rgb(160, 160, 175))),
            Span::styled(value, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ])
    };

    let mut lines = vec![
        Line::from(""),
        stat_line("Words Submitted:", format!("{}", s.words_submitted)),
        stat_line("Total Damage Dealt:", format!("{}", s.total_damage_dealt)),
        stat_line("Total Damage Taken:", format!("{}", s.total_damage_taken)),
        stat_line("Enemies Defeated:", format!("{}", s.enemies_defeated)),
        stat_line("Bosses Defeated:", format!("{}", s.bosses_defeated)),
        stat_line("Critical Hits:", format!("{}", s.crits_landed)),
        stat_line("Dodges:", format!("{}", s.dodges)),
        stat_line("Longest Word:", s.longest_word.clone()),
        stat_line("Highest Damage:", format!("{} ({})", s.highest_damage, s.highest_damage_word)),
        stat_line("Potions Used:", format!("{}", s.potions_used)),
        stat_line("Gems Earned:", format!("{}", s.gems_earned)),
        stat_line("Gems Spent:", format!("{}", s.gems_spent)),
    ];

    // Word history
    if !s.word_history.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Recent words:",
            Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD),
        )));
        let start = s.word_history.len().saturating_sub(8);
        let recent: Vec<String> = s.word_history[start..].to_vec();
        let history_str = recent.join(", ");
        lines.push(Line::from(Span::styled(
            format!("  {}", history_str),
            Style::default().fg(Color::Rgb(140, 200, 255)),
        )));
    }

    lines.push(Line::from(Span::styled(
        "  Press any key to close",
        Style::default().fg(Color::Rgb(100, 100, 120)),
    )));

    let text = Paragraph::new(lines).block(block);
    frame.render_widget(text, popup_area);
}
