use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

const TITLE_ART: &str = r#"
    ██╗     ███████╗██╗  ██╗██╗ ██████╗ ██████╗ ███╗   ██╗
    ██║     ██╔════╝╚██╗██╔╝██║██╔════╝██╔═══██╗████╗  ██║
    ██║     █████╗   ╚███╔╝ ██║██║     ██║   ██║██╔██╗ ██║
    ██║     ██╔══╝   ██╔██╗ ██║██║     ██║   ██║██║╚██╗██║
    ███████╗███████╗██╔╝ ██╗██║╚██████╗╚██████╔╝██║ ╚████║
    ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝
             ██████╗ ███████╗ █████╗ ██████╗ ███████╗██████╗
             ██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔════╝██╔══██╗
             ██████╔╝█████╗  ███████║██████╔╝█████╗  ██████╔╝
             ██╔══██╗██╔══╝  ██╔══██║██╔═══╝ ██╔══╝  ██╔══██╗
             ██║  ██║███████╗██║  ██║██║     ███████╗██║  ██║
             ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝
"#;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Top spacer
            Constraint::Length(14), // Title art
            Constraint::Length(2),  // Subtitle
            Constraint::Min(7),    // Menu
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Title art with gradient-like color
    let title_lines: Vec<Line> = TITLE_ART
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let color = if i < 7 {
                Color::Rgb(255, (80 + i * 10) as u8, (60 + i * 5) as u8)
            } else {
                Color::Rgb((200 + i * 3).min(255) as u8, (80 + i * 8).min(200) as u8, (60 + i * 6).min(180) as u8)
            };
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(color),
            ))
        })
        .collect();
    let title = Paragraph::new(title_lines).alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Subtitle
    let subtitle = Paragraph::new(vec![
        Line::from(Span::styled(
            "━━━━━━━ ⚔ A Word-Spelling RPG ⚔ ━━━━━━━",
            Style::default().fg(Color::Rgb(120, 100, 80)),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[2]);

    // Menu
    let mut menu_items = Vec::new();
    let mut idx = 0;

    if app.has_save {
        menu_items.push(menu_line("▸ Continue", app.menu_selection == idx, true));
        idx += 1;
    }

    menu_items.push(menu_line("▸ New Game", app.menu_selection == idx, false));
    idx += 1;
    menu_items.push(Line::from(""));
    menu_items.push(menu_line("▸ Quit", app.menu_selection == idx, false));

    let menu = Paragraph::new(menu_items).alignment(Alignment::Center);
    frame.render_widget(menu, chunks[3]);

    // Footer
    let footer = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Spell words. Slay monsters. Reap glory.",
            Style::default()
                .fg(Color::Rgb(100, 80, 70))
                .add_modifier(Modifier::ITALIC),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}

fn menu_line<'a>(text: &'a str, selected: bool, highlight: bool) -> Line<'a> {
    let base_color = if highlight {
        Color::Rgb(100, 200, 255)
    } else {
        Color::Rgb(200, 200, 210)
    };

    if selected {
        Line::from(vec![
            Span::styled(
                "  ≫ ",
                Style::default().fg(Color::Rgb(255, 220, 80)),
            ),
            Span::styled(
                text,
                Style::default()
                    .fg(Color::Rgb(255, 240, 150))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ≪  ",
                Style::default().fg(Color::Rgb(255, 220, 80)),
            ),
        ])
    } else {
        Line::from(Span::styled(
            format!("    {}    ", text),
            Style::default().fg(base_color),
        ))
    }
}

pub fn render_game_over(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Spacer
            Constraint::Length(5),  // Game over text
            Constraint::Length(2),  // Spacer
            Constraint::Length(8),  // Stats
            Constraint::Length(3),  // Prompt
            Constraint::Min(1),    // Footer
        ])
        .split(size);

    // Game over skull art
    let game_over = Paragraph::new(vec![
        Line::from(Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Rgb(120, 30, 30)),
        )),
        Line::from(Span::styled(
            "☠  G A M E   O V E R  ☠",
            Style::default()
                .fg(Color::Rgb(255, 50, 50))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Rgb(120, 30, 30)),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(game_over, chunks[1]);

    // Run stats
    let s = &app.stats;
    let stats = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Stage Reached:    ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(
                format!("{}", app.stage_number),
                Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Enemies Defeated: ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(format!("{}", s.enemies_defeated), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Words Spelled:    ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(format!("{}", s.words_submitted), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Longest Word:     ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(s.longest_word.clone(), Style::default().fg(Color::Rgb(100, 200, 255))),
        ]),
        Line::from(vec![
            Span::styled("  Best Hit:         ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(
                format!("{} dmg (\"{}\")", s.highest_damage, s.highest_damage_word),
                Style::default().fg(Color::Rgb(255, 120, 80)),
            ),
        ]),
        Line::from(""),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(stats, chunks[3]);

    // Continue prompt
    let prompt = Paragraph::new(Line::from(Span::styled(
        "Press Enter to return  │  Q to quit",
        Style::default().fg(Color::Rgb(100, 100, 120)),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(prompt, chunks[4]);
}
