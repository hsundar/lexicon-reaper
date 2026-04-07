use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::game::state::LogColor;

use super::icons;
use super::theme::theme_for_biome;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(" ≡ Combat Log ")
        .title_style(Style::default().fg(Color::Rgb(180, 180, 200)).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_lines = inner.height as usize;
    let start = if app.message_log.len() > max_lines {
        app.message_log.len() - max_lines
    } else {
        0
    };

    let lines: Vec<Line> = app.message_log[start..]
        .iter()
        .map(|entry| {
            let (prefix, color) = match entry.color {
                LogColor::Normal => (" · ".to_string(), Color::Rgb(180, 180, 190)),
                LogColor::PlayerDamage => (format!(" {} ", icons::SWORD), Color::Rgb(255, 120, 80)),
                LogColor::EnemyDamage => (format!(" {} ", icons::SKULL), Color::Rgb(255, 80, 80)),
                LogColor::Heal => (format!(" {} ", icons::HEART), Color::Rgb(80, 255, 120)),
                LogColor::Crit => (format!(" {} ", icons::BOLT), Color::Rgb(255, 230, 50)),
                LogColor::Info => (format!(" {} ", icons::DIAMOND), Color::Rgb(100, 180, 255)),
                LogColor::Warning => (format!(" {} ", icons::WARNING), Color::Rgb(255, 180, 50)),
            };
            Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::styled(
                    entry.text.clone(),
                    Style::default().fg(color),
                ),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}
