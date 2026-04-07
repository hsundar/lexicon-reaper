use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let player = &app.player;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(5),  // Weapon
            Constraint::Length(8),  // Books
            Constraint::Length(6),  // Potions
            Constraint::Min(3),    // Stats
            Constraint::Length(1), // Keybinds
        ])
        .split(size);

    // Header
    let header = Paragraph::new(Line::from(Span::styled(
        " INVENTORY ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Weapon
    let weapon_info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", player.weapon.name()),
                Style::default()
                    .fg(Color::Rgb(200, 150, 50))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Lv{}", player.weapon.level),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("  DMG Bonus: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("+{:.1}", player.weapon.damage_bonus()),
                Style::default().fg(Color::White),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Weapon "),
    );
    frame.render_widget(weapon_info, chunks[1]);

    // Books
    let mut book_lines = Vec::new();
    if player.books.is_empty() {
        book_lines.push(Line::from(Span::styled(
            "  No books equipped (buy from shop)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, book) in player.books.iter().enumerate() {
            let selected = app.inventory_selection == i;
            let prefix = if selected { "> " } else { "  " };
            book_lines.push(Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default().fg(if selected {
                        Color::Yellow
                    } else {
                        Color::White
                    }),
                ),
                Span::styled(
                    format!("{} ", book.name()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(if selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(
                    format!("Lv{}", book.level),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" (XP: {}/{})", book.xp, book.xp_to_next_level()),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }
    }
    let books = Paragraph::new(book_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Books ({}/3) ", player.books.len())),
    );
    frame.render_widget(books, chunks[2]);

    // Potions
    let mut potion_lines = Vec::new();
    if player.potions.is_empty() {
        potion_lines.push(Line::from(Span::styled(
            "  No potions (buy from shop)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for potion in &player.potions {
            potion_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    potion.name(),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }
    }
    let potions = Paragraph::new(potion_lines).block(
        Block::default().borders(Borders::ALL).title(format!(
            " Potions ({}/{}) ",
            player.potions.len(),
            player.max_potions
        )),
    );
    frame.render_widget(potions, chunks[3]);

    // Stats
    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  HP: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{}", player.hp, player.max_hp),
                Style::default().fg(Color::Green),
            ),
            Span::styled("  ATK: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", player.base_attack),
                Style::default().fg(Color::White),
            ),
            Span::styled("  ARM: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", player.armor),
                Style::default().fg(Color::White),
            ),
            Span::styled("  DODGE: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}%", (player.dodge_chance * 100.0) as i32),
                Style::default().fg(Color::White),
            ),
            Span::styled("  CRIT: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}%", (player.crit_chance * 100.0) as i32),
                Style::default().fg(Color::White),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Stats "));
    frame.render_widget(stats, chunks[4]);

    // Keybinds
    let keybinds = Line::from(vec![
        Span::styled(" Esc ", Style::default().fg(Color::Yellow)),
        Span::raw("Back "),
        Span::styled(" Del ", Style::default().fg(Color::Yellow)),
        Span::raw("Drop Book"),
    ]);
    frame.render_widget(Paragraph::new(keybinds), chunks[5]);
}
