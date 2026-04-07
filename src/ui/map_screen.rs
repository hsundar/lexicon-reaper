use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::generation::stage::Biome;

use super::icons;

/// Render the map screen showing stage progression
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(16),   // Map
            Constraint::Length(5), // Stage info
            Constraint::Length(1), // Keybinds
        ])
        .split(size);

    render_header(frame, chunks[0], app);
    render_map_path(frame, chunks[1], app);
    render_stage_info(frame, chunks[2], app);
    render_keybinds(frame, chunks[3]);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let biome = Biome::from_stage(app.stage_number);
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} JOURNEY MAP ", icons::STAR),
            Style::default()
                .fg(Color::Rgb(255, 220, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Stage {} ", app.stage_number),
            Style::default().fg(Color::White),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            biome.name(),
            Style::default().fg(biome_color(biome)),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(120, 100, 60))),
    );
    frame.render_widget(header, area);
}

fn render_map_path(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 100)))
        .title(" Path ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(160, 160, 180))
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let current = app.stage_number;
    // Show stages from (current - 2) to (current + 7), total of ~10
    let start = if current > 2 { current - 2 } else { 1 };
    let end = start + 9;

    let mut lines: Vec<Line> = Vec::new();

    // Biome header row
    lines.push(Line::from(""));

    // Build the path visualization
    // Row 1: biome labels
    let mut biome_spans: Vec<Span> = vec![Span::raw("  ")];
    for s in start..=end {
        let biome = Biome::from_stage(s);
        let color = biome_color(biome);
        let label = biome_short(biome);
        biome_spans.push(Span::styled(
            format!("{:^7}", label),
            Style::default().fg(color),
        ));
    }
    lines.push(Line::from(biome_spans));

    // Row 2: stage numbers
    let mut num_spans: Vec<Span> = vec![Span::raw("  ")];
    for s in start..=end {
        let is_current = s == current;
        let is_past = s < current;
        let is_boss = s % 10 == 0;

        let style = if is_current {
            Style::default()
                .fg(Color::Rgb(255, 255, 100))
                .add_modifier(Modifier::BOLD)
        } else if is_past {
            Style::default().fg(Color::Rgb(80, 80, 100))
        } else {
            Style::default().fg(Color::Rgb(160, 160, 180))
        };

        let label = if is_boss {
            format!(" [{:>2}] ", s)
        } else {
            format!("  {:>2}  ", s)
        };
        num_spans.push(Span::styled(label, style));
    }
    lines.push(Line::from(num_spans));

    // Row 3: the path line with node markers
    let mut path_spans: Vec<Span> = vec![Span::raw("  ")];
    for s in start..=end {
        let is_current = s == current;
        let is_past = s < current;
        let is_boss = s % 10 == 0;

        let (node, color) = if is_current {
            (format!("══{}══", icons::DIAMOND), Color::Rgb(255, 220, 80))
        } else if is_boss {
            if is_past {
                (format!("──{}──", icons::SKULL), Color::Rgb(80, 80, 100))
            } else {
                (format!("──{}──", icons::SKULL), Color::Rgb(255, 80, 80))
            }
        } else if is_past {
            ("──●──".to_string(), Color::Rgb(80, 80, 100))
        } else {
            ("──○──".to_string(), Color::Rgb(120, 120, 140))
        };

        path_spans.push(Span::styled(
            format!("{:^7}", node),
            Style::default().fg(color),
        ));
    }
    lines.push(Line::from(path_spans));

    // Row 4: connectors
    let mut conn_spans: Vec<Span> = vec![Span::raw("  ")];
    for s in start..=end {
        let is_past = s < current;
        let color = if is_past {
            Color::Rgb(80, 80, 100)
        } else {
            Color::Rgb(120, 120, 140)
        };

        // Show connector to next
        if s < end {
            conn_spans.push(Span::styled(
                "───────",
                Style::default().fg(color),
            ));
        } else {
            conn_spans.push(Span::styled("  ···  ", Style::default().fg(Color::Rgb(60, 60, 80))));
        }
    }
    lines.push(Line::from(conn_spans));

    // Row 5: enemy count preview for upcoming stages
    lines.push(Line::from(""));
    let mut preview_spans: Vec<Span> = vec![Span::raw("  ")];
    for s in start..=end {
        let is_current = s == current;
        let is_past = s < current;
        let is_boss = s % 10 == 0;

        let label = if is_past {
            "  ✓   ".to_string()
        } else if is_boss {
            format!(" {}BOSS ", icons::STAR)
        } else if is_current {
            " HERE ".to_string()
        } else {
            let enemy_count = if s <= 3 {
                1
            } else if s <= 10 {
                1 // show minimum
            } else {
                2
            };
            format!(" {}x{}  ", enemy_count, icons::SKULL)
        };

        let color = if is_current {
            Color::Rgb(255, 220, 80)
        } else if is_past {
            Color::Rgb(60, 100, 60)
        } else if is_boss {
            Color::Rgb(255, 80, 80)
        } else {
            Color::Rgb(140, 140, 160)
        };

        preview_spans.push(Span::styled(
            format!("{:^7}", label),
            Style::default().fg(color),
        ));
    }
    lines.push(Line::from(preview_spans));

    // Biome transition markers
    lines.push(Line::from(""));
    let mut transition_lines: Vec<Span> = vec![Span::styled(
        "  Biomes: ",
        Style::default()
            .fg(Color::Rgb(140, 140, 160))
            .add_modifier(Modifier::BOLD),
    )];
    let biomes = [Biome::Forest, Biome::Crypt, Biome::Volcano, Biome::Abyss, Biome::Void];
    let current_biome = Biome::from_stage(current);
    for (i, biome) in biomes.iter().enumerate() {
        if i > 0 {
            transition_lines.push(Span::styled(" → ", Style::default().fg(Color::Rgb(80, 80, 100))));
        }
        let is_active = *biome == current_biome;
        let style = if is_active {
            Style::default()
                .fg(biome_color(*biome))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(80, 80, 100))
        };
        transition_lines.push(Span::styled(biome.name(), style));
    }
    lines.push(Line::from(transition_lines));

    // Legend
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("Cleared  ", Style::default().fg(Color::Rgb(120, 120, 140))),
        Span::styled("○ ", Style::default().fg(Color::Rgb(120, 120, 140))),
        Span::styled("Upcoming  ", Style::default().fg(Color::Rgb(120, 120, 140))),
        Span::styled(format!("{} ", icons::DIAMOND), Style::default().fg(Color::Rgb(255, 220, 80))),
        Span::styled("You  ", Style::default().fg(Color::Rgb(120, 120, 140))),
        Span::styled(format!("{} ", icons::SKULL), Style::default().fg(Color::Rgb(255, 80, 80))),
        Span::styled("Boss  ", Style::default().fg(Color::Rgb(120, 120, 140))),
        Span::styled("[N] ", Style::default().fg(Color::Rgb(160, 160, 180))),
        Span::styled("Boss stage", Style::default().fg(Color::Rgb(120, 120, 140))),
    ]));

    let map_widget = Paragraph::new(lines);
    frame.render_widget(map_widget, inner);
}

fn render_stage_info(frame: &mut Frame, area: Rect, app: &App) {
    let next_stage = app.stage_number + 1;
    let next_biome = Biome::from_stage(next_stage);
    let is_boss = next_stage % 10 == 0;
    let biome_change = Biome::from_stage(app.stage_number) != next_biome;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                " Next: ",
                Style::default()
                    .fg(Color::Rgb(180, 180, 200))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Stage {} ", next_stage),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("in ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(
                next_biome.name(),
                Style::default()
                    .fg(biome_color(next_biome))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    if is_boss {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} BOSS STAGE! Prepare yourself!", icons::WARNING),
                Style::default()
                    .fg(Color::Rgb(255, 80, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    if biome_change {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} Entering new biome: {}", icons::STAR, next_biome.name()),
                Style::default().fg(biome_color(next_biome)),
            ),
        ]));
    }

    let info = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 80, 100)))
            .title(" Ahead ")
            .title_style(Style::default().fg(Color::Rgb(160, 160, 180))),
    );
    frame.render_widget(info, area);
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let sep = Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 70)));
    let keybinds = Line::from(vec![
        Span::styled(" Enter", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Shop", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("C", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Continue to next stage", Style::default().fg(Color::Rgb(180, 180, 190))),
        sep.clone(),
        Span::styled("Q", Style::default().fg(Color::Rgb(255, 200, 80))),
        Span::styled(" Quit", Style::default().fg(Color::Rgb(180, 180, 190))),
    ]);
    frame.render_widget(Paragraph::new(keybinds), area);
}

fn biome_color(biome: Biome) -> Color {
    match biome {
        Biome::Forest => Color::Rgb(100, 200, 100),
        Biome::Crypt => Color::Rgb(160, 160, 200),
        Biome::Volcano => Color::Rgb(255, 120, 50),
        Biome::Abyss => Color::Rgb(180, 80, 220),
        Biome::Void => Color::Rgb(200, 200, 200),
    }
}

fn biome_short(biome: Biome) -> &'static str {
    match biome {
        Biome::Forest => "Forest",
        Biome::Crypt => "Crypt",
        Biome::Volcano => "Volcan",
        Biome::Abyss => "Abyss",
        Biome::Void => "Void",
    }
}
