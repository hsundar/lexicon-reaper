use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::game::damage::estimate_damage;

use super::icons;
use super::theme::theme_for_biome;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(format!(" {} Word ", icons::BOOK))
        .title_style(Style::default().fg(Color::Rgb(180, 220, 255)).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let combat = match &app.combat {
        Some(c) => c,
        None => return,
    };

    let board = &combat.board;
    let word = board.selected_word();
    let selection_count = board.selection_count();

    let mut lines: Vec<Line> = Vec::new();

    if selection_count == 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Select tiles to form a word",
            Style::default().fg(Color::Rgb(100, 100, 120)),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Use Space to pick letters",
            Style::default().fg(Color::Rgb(80, 80, 100)),
        )));
        lines.push(Line::from(Span::styled(
            "  Press Enter to submit",
            Style::default().fg(Color::Rgb(80, 80, 100)),
        )));
    } else {
        // Current word - large display
        lines.push(Line::from(vec![
            Span::styled(" » ", Style::default().fg(Color::Rgb(255, 200, 80))),
            Span::styled(
                &word,
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({} letters)", word.len()),
                Style::default().fg(Color::Rgb(120, 120, 140)),
            ),
        ]));

        // Validity check with visual indicator
        let is_valid = word.len() >= 3 && app.dictionary.is_word(&word);
        let is_prefix = app.dictionary.is_prefix(&word);

        let validity_line = if is_valid {
            Line::from(vec![
                Span::styled(format!("   {} ", icons::CHECK), Style::default().fg(Color::Rgb(80, 255, 80))),
                Span::styled(
                    "VALID WORD",
                    Style::default()
                        .fg(Color::Rgb(80, 255, 80))
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else if is_prefix && word.len() >= 2 {
            Line::from(vec![
                Span::styled("   ◌ ", Style::default().fg(Color::Rgb(255, 200, 80))),
                Span::styled(
                    "building...",
                    Style::default().fg(Color::Rgb(255, 200, 80)),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(format!("   {} ", icons::CROSS), Style::default().fg(Color::Rgb(255, 80, 80))),
                Span::styled(
                    "INVALID",
                    Style::default()
                        .fg(Color::Rgb(255, 80, 80))
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        };
        lines.push(validity_line);

        // Estimated damage
        if is_valid {
            let enemy = combat.current_enemy();
            let est_damage = estimate_damage(board, &app.player, enemy);

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(format!("   {} Damage: ", icons::SWORD), Style::default().fg(Color::Rgb(180, 180, 190))),
                Span::styled(
                    format!("~{}", est_damage),
                    Style::default()
                        .fg(Color::Rgb(255, 100, 80))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));

            // Constraint check
            if let Some(constraint) = &enemy.word_constraint {
                let passes = constraint.check(&word);
                if passes {
                    lines.push(Line::from(vec![
                        Span::styled(format!("   {} ", icons::CHECK), Style::default().fg(Color::Rgb(80, 255, 80))),
                        Span::styled(
                            "Constraint met",
                            Style::default().fg(Color::Rgb(80, 255, 80)),
                        ),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(format!("   {} ", icons::CROSS), Style::default().fg(Color::Rgb(255, 80, 80))),
                        Span::styled(
                            "Constraint FAILED",
                            Style::default()
                                .fg(Color::Rgb(255, 80, 80))
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
            }
        }
    }

    // Tile effect legend (compact)
    if selection_count == 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ─── Tile Effects ───",
            Style::default().fg(Color::Rgb(100, 100, 120)),
        )));
        let legend_items = [
            (icons::CRYSTAL, "Crystal", Color::Rgb(80, 255, 255)),
            (icons::POISON, "Poison", Color::Rgb(80, 255, 80)),
            (icons::SPIKE, "Spike", Color::Rgb(255, 80, 80)),
            (icons::STONE, "Stone", Color::Rgb(140, 140, 140)),
        ];
        for (sym, name, color) in legend_items {
            lines.push(Line::from(vec![
                Span::styled(format!("   {} ", sym), Style::default().fg(color)),
                Span::styled(name, Style::default().fg(Color::Rgb(160, 160, 175))),
            ]));
        }
    }

    // Equipment summary at bottom
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ─── Equipment ───",
        Style::default().fg(Color::Rgb(100, 100, 120)),
    )));
    let player = &app.player;
    lines.push(Line::from(vec![
        Span::styled(format!("  {} ", icons::SWORD), Style::default().fg(Color::Rgb(200, 150, 50))),
        Span::styled(
            format!("{} Lv{}", player.weapon.name(), player.weapon.level),
            Style::default().fg(Color::Rgb(230, 180, 80)),
        ),
    ]));
    if !player.books.is_empty() {
        let mut book_spans: Vec<Span> = vec![Span::styled(format!("  {} ", icons::BOOK), Style::default().fg(Color::Rgb(120, 180, 255)))];
        for (i, book) in player.books.iter().enumerate() {
            if i > 0 {
                book_spans.push(Span::styled("·", Style::default().fg(Color::DarkGray)));
            }
            book_spans.push(Span::styled(
                format!("{}{}", book.short_name(), book.level),
                Style::default().fg(Color::Rgb(140, 200, 255)),
            ));
        }
        lines.push(Line::from(book_spans));
    }
    if !player.potions.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", icons::FLASK), Style::default().fg(Color::Rgb(200, 120, 255))),
            Span::styled(
                format!("{}x potions", player.potions.len()),
                Style::default().fg(Color::Rgb(200, 160, 255)),
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}
