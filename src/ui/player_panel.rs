use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::app::App;

use super::icons;
use super::theme::theme_for_biome;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(format!(" {} Reaper ", icons::SWORD))
        .title_style(Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let player = &app.player;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // HP bar
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Stats
            Constraint::Length(1), // Weapon
            Constraint::Length(1), // Books
            Constraint::Length(1), // Potions
            Constraint::Length(1), // Gems
        ])
        .split(inner);

    // HP bar
    let hp_ratio = player.hp as f64 / player.max_hp as f64;
    let hp_color = if hp_ratio > 0.6 {
        Color::Rgb(80, 220, 80)
    } else if hp_ratio > 0.3 {
        Color::Rgb(255, 200, 50)
    } else {
        Color::Rgb(255, 60, 60)
    };
    let hp_label = format!("{} {}/{}", icons::HEART, player.hp, player.max_hp);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(hp_color).bg(Color::Rgb(40, 40, 50)))
        .ratio(hp_ratio.clamp(0.0, 1.0))
        .label(hp_label);
    frame.render_widget(gauge, chunks[0]);

    // Stats with icons
    let stats = Line::from(vec![
        Span::styled(format!("{} ", icons::SWORD), Style::default().fg(Color::Rgb(255, 120, 80))),
        Span::styled(
            format!("{}", player.base_attack),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {} ", icons::SHIELD), Style::default().fg(Color::Rgb(120, 160, 255))),
        Span::styled(
            format!("{}", player.armor),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ↺ ", Style::default().fg(Color::Rgb(100, 220, 100))),
        Span::styled(
            format!("{}%", (player.dodge_chance * 100.0) as i32),
            Style::default().fg(Color::White),
        ),
        Span::styled(format!("  {}", icons::BOLT), Style::default().fg(Color::Rgb(255, 220, 80))),
        Span::styled(
            format!("{}%", (player.crit_chance * 100.0) as i32),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(stats), chunks[2]);

    // Weapon
    let weapon_line = Line::from(vec![
        Span::styled(format!("{} ", icons::SWORD), Style::default().fg(Color::Rgb(200, 150, 50))),
        Span::styled(
            player.weapon.name(),
            Style::default()
                .fg(Color::Rgb(230, 180, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" Lv{}", player.weapon.level),
            Style::default().fg(Color::Rgb(255, 220, 100)),
        ),
    ]);
    frame.render_widget(Paragraph::new(weapon_line), chunks[3]);

    // Books
    let book_spans: Vec<Span> = if player.books.is_empty() {
        vec![Span::styled(
            "  No books equipped",
            Style::default().fg(Color::Rgb(80, 80, 90)),
        )]
    } else {
        let mut spans = vec![Span::styled(format!("{} ", icons::BOOK), Style::default().fg(Color::Rgb(120, 180, 255)))];
        for (i, book) in player.books.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" · ", Style::default().fg(Color::DarkGray)));
            }
            spans.push(Span::styled(
                format!("{}{}", book.short_name(), book.level),
                Style::default().fg(Color::Rgb(140, 200, 255)),
            ));
        }
        spans
    };
    frame.render_widget(Paragraph::new(Line::from(book_spans)), chunks[4]);

    // Potions
    let potion_line = if player.potions.is_empty() {
        Line::from(Span::styled(
            "  No potions",
            Style::default().fg(Color::Rgb(80, 80, 90)),
        ))
    } else {
        let mut spans = vec![Span::styled(format!("{} ", icons::FLASK), Style::default().fg(Color::Rgb(200, 120, 255)))];
        for (i, p) in player.potions.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(", ", Style::default().fg(Color::DarkGray)));
            }
            let color = match p {
                crate::game::equipment::Potion::Healing { .. } => Color::Rgb(100, 255, 100),
                crate::game::equipment::Potion::Purity => Color::Rgb(200, 200, 255),
                crate::game::equipment::Potion::Lifesteal => Color::Rgb(255, 100, 100),
            };
            spans.push(Span::styled(p.name(), Style::default().fg(color)));
        }
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(potion_line), chunks[5]);

    // Gems
    let gems_line = Line::from(vec![
        Span::styled(format!("{} ", icons::DIAMOND), Style::default().fg(Color::Rgb(255, 215, 0))),
        Span::styled(
            format!("{}", player.gems),
            Style::default()
                .fg(Color::Rgb(255, 230, 100))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" gems", Style::default().fg(Color::Rgb(180, 160, 80))),
    ]);
    frame.render_widget(Paragraph::new(gems_line), chunks[6]);
}
