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
        .title(format!(" {} Enemy ", icons::SKULL))
        .title_style(Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let combat = match &app.combat {
        Some(c) => c,
        None => return,
    };

    let enemy = combat.current_enemy();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // ASCII art
            Constraint::Length(1), // Name + enemy count
            Constraint::Length(1), // HP bar
            Constraint::Length(1), // Constraint
        ])
        .split(inner);

    // ASCII art with biome color
    let art = get_enemy_art(&enemy.sprite_key);
    let art_color = if enemy.is_boss {
        Color::Rgb(255, 80, 80)
    } else {
        theme.enemy_art
    };
    let art_paragraph = Paragraph::new(art).style(Style::default().fg(art_color));
    frame.render_widget(art_paragraph, chunks[0]);

    // Name + enemy progress
    let enemy_progress = if combat.enemies.len() > 1 {
        format!(
            " ({}/{})",
            combat.current_enemy_idx + 1,
            combat.enemies.len()
        )
    } else {
        String::new()
    };

    let name_style = if enemy.is_boss {
        Style::default()
            .fg(Color::Rgb(255, 80, 80))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };

    let name = Paragraph::new(Line::from(vec![
        Span::styled(&enemy.name, name_style),
        if enemy.is_boss {
            Span::styled(
                format!(" {}BOSS{}", icons::STAR, icons::STAR),
                Style::default()
                    .fg(Color::Rgb(255, 200, 50))
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
        Span::styled(
            enemy_progress,
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(name, chunks[1]);

    // HP bar with visual flair
    let hp_ratio = enemy.hp as f64 / enemy.max_hp as f64;
    let hp_color = if hp_ratio > 0.6 {
        Color::Rgb(80, 220, 80)
    } else if hp_ratio > 0.3 {
        Color::Rgb(255, 200, 50)
    } else {
        Color::Rgb(255, 60, 60)
    };

    let hp_label = format!("{} {}/{}", icons::HEART, enemy.hp, enemy.max_hp);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(hp_color).bg(Color::Rgb(40, 40, 50)))
        .ratio(hp_ratio.clamp(0.0, 1.0))
        .label(hp_label);
    frame.render_widget(gauge, chunks[2]);

    // Word constraint
    if let Some(constraint) = &enemy.word_constraint {
        let constraint_text = Paragraph::new(Line::from(vec![
            Span::styled(format!("{} ", icons::WARNING), Style::default().fg(Color::Rgb(255, 200, 50))),
            Span::styled(
                constraint.description(),
                Style::default()
                    .fg(Color::Rgb(255, 200, 50))
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            ),
        ]));
        frame.render_widget(constraint_text, chunks[3]);
    }
}

fn get_enemy_art(sprite_key: &str) -> String {
    match sprite_key {
        "goblin" => r#"     /\_/\
    ( o.o )
    (> ^ <)
     /| |\
    (_| |_)"#
            .to_string(),
        "skeleton" => r#"     _☠_
    /o.o\
    |=+=|
    /| |\
   (_/ \_)"#
            .to_string(),
        "wolf" => r#"   /\    /\
  /  \../  \
 ( ◉    ◉  )
  \  <▽>  /
   '------'"#
            .to_string(),
        "slime" => r#"    .-"""-.
   /  o  o \
  |    __   |
   \  \__/ /
    '-..-'"#
            .to_string(),
        "boss_goblin" => r#"  ♛ ___/\___
   / ◉    ◉ \
  (  >>==<<  )
   \_|_/\_|_/
     |    |
    /|    |\"#
            .to_string(),
        "boss_skeleton" => r#"  ♛  _☠☠_
    /◉  ◉\
   |==++=+|
    \_||_/
    /|  |\
   (_/  \_)"#
            .to_string(),
        "boss_wolf" => r#"  ♛/\      /\
   /  \=*=/  \
  ( ◉◉    ◉◉ )
   \  <▽▽>  /
    '-======-'"#
            .to_string(),
        "boss_slime" => r#"  ♛.-"""""-.
   / ◉    ◉ \
  |  /‾‾‾\   |
  |  \___/   |
   \_________/"#
            .to_string(),
        _ => r#"    ??????
    ?    ?
    ?    ?
    ??????
    ? ?? ?"#
            .to_string(),
    }
}
