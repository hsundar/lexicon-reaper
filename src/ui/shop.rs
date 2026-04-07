use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::game::shop::{can_buy, cost_for_item};

use super::icons;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(10),  // Shop items
            Constraint::Length(6), // Player info
            Constraint::Length(1), // Keybinds
        ])
        .split(size);

    // Header
    let shop_state = match &app.shop_state {
        Some(s) => s,
        None => return,
    };

    let mut header_lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} REAPER'S SHOP {} ", icons::DIAMOND, icons::DIAMOND),
                Style::default()
                    .fg(Color::Rgb(255, 215, 0))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" Stage {} Complete! ", app.stage_number),
                Style::default().fg(Color::Rgb(180, 180, 200)),
            ),
            Span::styled(format!("  {} ", icons::DIAMOND), Style::default().fg(Color::Rgb(255, 215, 0))),
            Span::styled(
                format!("{} gems", app.player.gems),
                Style::default()
                    .fg(Color::Rgb(255, 230, 100))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    if let Some(msg) = &shop_state.message {
        header_lines.push(Line::from(Span::styled(
            format!(" → {}", msg),
            Style::default().fg(Color::Rgb(100, 255, 150)),
        )));
    }

    let header = Paragraph::new(header_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(180, 150, 50))),
    );
    frame.render_widget(header, chunks[0]);

    // Shop items
    let items: Vec<ListItem> = shop_state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let cost = cost_for_item(item, &app.player);
            let affordable = can_buy(item, &app.player);
            let selected = i == shop_state.selected_index;

            let prefix = if selected { " ▸ " } else { "   " };
            let cost_str = format!("  {} gems", cost);

            let name_color = if !affordable {
                Color::Rgb(80, 80, 90)
            } else if selected {
                Color::Rgb(255, 230, 100)
            } else {
                Color::Rgb(200, 200, 210)
            };

            let cost_color = if !affordable {
                Color::Rgb(200, 60, 60)
            } else {
                Color::Rgb(100, 220, 100)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default().fg(if selected {
                            Color::Rgb(255, 215, 0)
                        } else {
                            Color::DarkGray
                        }),
                    ),
                    Span::styled(
                        item.name(),
                        Style::default()
                            .fg(name_color)
                            .add_modifier(if selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                    Span::styled(cost_str, Style::default().fg(cost_color)),
                ]),
                Line::from(vec![
                    Span::raw("     "),
                    Span::styled(
                        item.description(),
                        Style::default().fg(Color::Rgb(120, 120, 140)),
                    ),
                ]),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(100, 100, 120)))
            .title(" Buy & Upgrade ")
            .title_style(Style::default().fg(Color::Rgb(180, 180, 200)).add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(list, chunks[1]);

    // Player info summary
    let player = &app.player;
    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(format!(" {} ", icons::HEART), Style::default().fg(Color::Rgb(80, 220, 80))),
            Span::styled(
                format!("{}/{}", player.hp, player.max_hp),
                Style::default().fg(Color::Rgb(80, 220, 80)),
            ),
            Span::styled(format!("  {} ", icons::SWORD), Style::default().fg(Color::Rgb(255, 120, 80))),
            Span::styled(
                format!("{}", player.base_attack),
                Style::default().fg(Color::White),
            ),
            Span::styled(format!("  {} ", icons::SHIELD), Style::default().fg(Color::Rgb(120, 160, 255))),
            Span::styled(
                format!("{}", player.armor),
                Style::default().fg(Color::White),
            ),
            Span::styled("  ↺ ", Style::default().fg(Color::Rgb(100, 220, 100))),
            Span::styled(
                format!("{}%", (player.dodge_chance * 100.0) as i32),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!(" {} ", icons::SWORD), Style::default().fg(Color::Rgb(230, 180, 80))),
            Span::styled(
                format!("{} Lv{}", player.weapon.name(), player.weapon.level),
                Style::default().fg(Color::Rgb(230, 180, 80)),
            ),
            Span::styled(format!("  {} ", icons::FLASK), Style::default().fg(Color::Rgb(200, 120, 255))),
            Span::styled(
                format!("{}/{}", player.potions.len(), player.max_potions),
                Style::default().fg(Color::Rgb(200, 160, 255)),
            ),
            Span::styled(format!("  {} ", icons::BOOK), Style::default().fg(Color::Rgb(120, 180, 255))),
            Span::styled(
                format!("{}/3", player.books.len()),
                Style::default().fg(Color::Rgb(140, 200, 255)),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(100, 100, 120)))
            .title(" Stats ")
            .title_style(Style::default().fg(Color::Rgb(180, 180, 200))),
    );
    frame.render_widget(info, chunks[2]);

    // Keybinds
    let sep = Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 70)));
    let keybinds = Line::from(vec![
        Span::styled(" ↑↓", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Navigate", Style::default().fg(Color::Rgb(160, 160, 175))),
        sep.clone(),
        Span::styled("Enter", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Buy", Style::default().fg(Color::Rgb(160, 160, 175))),
        sep.clone(),
        Span::styled("C", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Continue", Style::default().fg(Color::Rgb(160, 160, 175))),
        sep.clone(),
        Span::styled("I", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Inventory", Style::default().fg(Color::Rgb(160, 160, 175))),
    ]);
    frame.render_widget(Paragraph::new(keybinds), chunks[3]);
}
