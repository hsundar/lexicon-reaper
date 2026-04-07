use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use tui_big_text::{BigText, PixelSize};

use crate::app::App;
use crate::game::board::{BOARD_COLS, BOARD_ROWS};
use crate::game::state::CombatPhase;
use crate::game::tile::{TileEffect, TileRarity};

use super::icons;
use super::theme::theme_for_biome;

pub const TILE_WIDTH: u16 = 12;
pub const TILE_HEIGHT: u16 = 8;
const TRAY_TILE_W: u16 = 8;
const TRAY_TILE_H: u16 = 6;

/// Render the full-width word tray (called from layout as a separate row)
pub fn render_tray(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(format!(" {} Word Tray ", icons::BOOK))
        .title_style(
            Style::default()
                .fg(Color::Rgb(180, 220, 255))
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let combat = match &app.combat {
        Some(c) => c,
        None => return,
    };

    render_word_tray(frame, inner, app, combat);
}

/// Render the letter grid only (called from layout as a separate panel)
pub fn render_grid(frame: &mut Frame, area: Rect, app: &App) {
    let theme = theme_for_biome(app.biome);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(format!(" {} Board ", icons::DIAMOND))
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let combat = match &app.combat {
        Some(c) => c,
        None => return,
    };

    let is_input = matches!(combat.phase, CombatPhase::PlayerInput);
    render_letter_grid(frame, inner, app, combat, is_input);
}

/// Render the word tray showing selected letters as a Scrabble-like rack
fn render_word_tray(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
) {
    let board = &combat.board;
    let positions = board.selected_positions();
    let word = board.selected_word();

    if positions.is_empty() {
        // Empty tray: show placeholder
        let placeholder = "Select letters to spell a word";
        let ph_style = Style::default().fg(Color::Rgb(80, 80, 100));
        let ph_rect = Rect::new(
            area.x + (area.width.saturating_sub(placeholder.len() as u16)) / 2,
            area.y + area.height / 2,
            placeholder.len() as u16,
            1,
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(placeholder, ph_style))),
            ph_rect,
        );
        // Draw tray baseline
        let base_rect = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "▁".repeat(area.width as usize),
                Style::default().fg(Color::Rgb(60, 60, 80)),
            ))),
            base_rect,
        );
        return;
    }

    // Validity indicator
    let is_valid = word.len() >= 3 && app.dictionary.is_word(&word);
    let is_prefix = word.len() >= 2 && app.dictionary.is_prefix(&word);
    let tray_border_color = if is_valid {
        Color::Rgb(80, 220, 80)
    } else if is_prefix {
        Color::Rgb(200, 180, 60)
    } else {
        Color::Rgb(180, 60, 60)
    };

    // Center the tray tiles
    let tray_total_w = positions.len() as u16 * TRAY_TILE_W;
    let tray_x = area.x + (area.width.saturating_sub(tray_total_w)) / 2;

    for (i, (r, c)) in positions.iter().enumerate() {
        let tile = board.tile(*r, *c);
        let tx = tray_x + i as u16 * TRAY_TILE_W;

        if tx + TRAY_TILE_W > area.x + area.width {
            break;
        }

        let tile_rect = Rect::new(tx, area.y, TRAY_TILE_W, TRAY_TILE_H);

        let (fg, bg) = tray_tile_colors(tile.rarity, tile.effect);
        let style = Style::default().fg(fg).bg(bg);

        // Fill background
        let fill = " ".repeat(TRAY_TILE_W as usize);
        for dy in 0..TRAY_TILE_H {
            let lr = Rect::new(tx, area.y + dy, TRAY_TILE_W, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(&fill, style))),
                lr,
            );
        }

        // Top border line with validity color
        let border_rect = Rect::new(tx, area.y, TRAY_TILE_W, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "▔".repeat(TRAY_TILE_W as usize),
                Style::default().fg(tray_border_color).bg(bg),
            ))),
            border_rect,
        );

        // BigText letter centered in tray tile
        let letter_str = tile.letter.to_string();
        let big_text = BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::default().fg(fg).bg(bg))
            .centered()
            .lines(vec![Line::from(letter_str)])
            .build();

        let letter_rect = Rect::new(tx, area.y + 1, TRAY_TILE_W, 4);
        frame.render_widget(big_text, letter_rect);

        // Bottom: rarity dots
        let dots = match tile.rarity {
            TileRarity::Bronze => format!(" {}", icons::STAR),
            TileRarity::Silver => format!("{}{}", icons::STAR, icons::STAR),
            TileRarity::Gold => format!("{}{}{}", icons::STAR, icons::STAR, icons::STAR),
        };
        let dot_rect = Rect::new(tx, area.y + TRAY_TILE_H - 1, TRAY_TILE_W, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{:^w$}", dots, w = TRAY_TILE_W as usize),
                Style::default().fg(rarity_dot_color(tile.rarity)).bg(bg),
            ))),
            dot_rect,
        );
    }
}

/// Render the 3x5 letter grid. Selected tiles show as blank/empty.
fn render_letter_grid(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    combat: &crate::game::state::CombatState,
    is_input: bool,
) {
    let board = &combat.board;
    let cursor = combat.cursor;

    let grid_w = BOARD_COLS as u16 * TILE_WIDTH;
    let grid_h = BOARD_ROWS as u16 * TILE_HEIGHT;
    let offset_x = (area.width.saturating_sub(grid_w)) / 2;
    let offset_y = (area.height.saturating_sub(grid_h)) / 2;

    for row in 0..BOARD_ROWS {
        for col in 0..BOARD_COLS {
            let tile = board.tile(row, col);
            let x = area.x + offset_x + (col as u16) * TILE_WIDTH;
            let y = area.y + offset_y + (row as u16) * TILE_HEIGHT;

            if x + TILE_WIDTH > area.x + area.width || y + TILE_HEIGHT > area.y + area.height {
                continue;
            }

            let is_cursor = is_input && row == cursor.0 && col == cursor.1;

            if tile.selected {
                // Selected tile: render as empty/blank slot
                render_empty_slot(frame, x, y, is_cursor);
            } else {
                // Normal tile: render with BigText letter
                render_tile(frame, x, y, tile, is_cursor);
            }
        }
    }
}

/// Render a blank slot where a selected tile was taken from
fn render_empty_slot(frame: &mut Frame, x: u16, y: u16, is_cursor: bool) {
    let bg = if is_cursor {
        Color::Rgb(60, 60, 70)
    } else {
        Color::Rgb(30, 30, 40)
    };
    let fg = Color::Rgb(60, 60, 80);
    let style = Style::default().fg(fg).bg(bg);

    let fill = " ".repeat(TILE_WIDTH as usize);
    for dy in 0..TILE_HEIGHT {
        let lr = Rect::new(x, y + dy, TILE_WIDTH, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(&fill, style))),
            lr,
        );
    }

    // Subtle border to show the empty slot
    let top = Rect::new(x, y, TILE_WIDTH, 1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("{:·^w$}", "", w = TILE_WIDTH as usize),
            style,
        ))),
        top,
    );
    let bot = Rect::new(x, y + TILE_HEIGHT - 1, TILE_WIDTH, 1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("{:·^w$}", "", w = TILE_WIDTH as usize),
            style,
        ))),
        bot,
    );
}

/// Render a normal (unselected) tile with BigText letter
fn render_tile(
    frame: &mut Frame,
    x: u16,
    y: u16,
    tile: &crate::game::tile::Tile,
    is_cursor: bool,
) {
    let (fg, bg) = tile_colors(tile.rarity, tile.effect);

    let base_style = if is_cursor {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(fg).bg(bg)
    };

    // Fill background
    let bg_fill = " ".repeat(TILE_WIDTH as usize);
    for dy in 0..TILE_HEIGHT {
        let lr = Rect::new(x, y + dy, TILE_WIDTH, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(&bg_fill, base_style))),
            lr,
        );
    }

    // Line 1: effect icon (right-aligned)
    let top_rect = Rect::new(x, y, TILE_WIDTH, 1);
    let effect_icon = effect_nerd_icon(tile.effect);
    let effect_style = if is_cursor {
        base_style
    } else if tile.effect != TileEffect::Normal {
        Style::default()
            .fg(effect_indicator_color(tile.effect))
            .bg(bg)
    } else {
        base_style
    };
    let pad_w = TILE_WIDTH as usize - 1;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ".repeat(pad_w), base_style),
            Span::styled(effect_icon, effect_style),
        ])),
        top_rect,
    );

    // Lines 2-5: BigText letter
    if tile.effect == TileEffect::Stone {
        for dy in 1..=5 {
            let sr = Rect::new(x, y + dy, TILE_WIDTH, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "░".repeat(TILE_WIDTH as usize),
                    base_style,
                ))),
                sr,
            );
        }
    } else {
        let letter_fg = if is_cursor { Color::Black } else { fg };
        let letter_bg = if is_cursor { Color::Rgb(255, 255, 255) } else { bg };

        let big_text = BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::default().fg(letter_fg).bg(letter_bg))
            .centered()
            .lines(vec![Line::from(tile.letter.to_string())])
            .build();

        let letter_rect = Rect::new(x + 1, y + 2, TILE_WIDTH - 2, 4);
        frame.render_widget(big_text, letter_rect);
    }

    // Line 7: rarity stars + timer
    let bottom_rect = Rect::new(x, y + TILE_HEIGHT - 2, TILE_WIDTH, 1);
    let rarity_str = match tile.rarity {
        TileRarity::Bronze => format!(" {} ", icons::STAR),
        TileRarity::Silver => format!("{}{} ", icons::STAR, icons::STAR),
        TileRarity::Gold => format!("{}{}{}", icons::STAR, icons::STAR, icons::STAR),
    };
    let rarity_style = if is_cursor {
        base_style
    } else {
        Style::default().fg(rarity_dot_color(tile.rarity)).bg(bg)
    };
    let timer_str = tile
        .timer
        .map(|t| format!("{}", t))
        .unwrap_or_else(|| " ".to_string());
    let timer_style = if is_cursor {
        base_style
    } else {
        Style::default().fg(Color::Rgb(200, 200, 100)).bg(bg)
    };
    let pad_bottom = TILE_WIDTH as usize - 5;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(rarity_str, rarity_style),
            Span::styled(" ".repeat(pad_bottom), base_style),
            Span::styled(timer_str, timer_style),
        ])),
        bottom_rect,
    );
}

fn tray_tile_colors(rarity: TileRarity, effect: TileEffect) -> (Color, Color) {
    // Tray tiles are brighter than grid tiles
    match effect {
        TileEffect::Crystal => (Color::Rgb(240, 255, 255), Color::Rgb(30, 130, 150)),
        TileEffect::Poison => (Color::Rgb(240, 255, 240), Color::Rgb(35, 130, 40)),
        TileEffect::Spike => (Color::Rgb(255, 235, 235), Color::Rgb(170, 50, 50)),
        _ => match rarity {
            TileRarity::Bronze => (Color::Rgb(255, 255, 255), Color::Rgb(85, 80, 105)),
            TileRarity::Silver => (Color::Rgb(255, 255, 255), Color::Rgb(65, 80, 160)),
            TileRarity::Gold => (Color::Rgb(255, 255, 240), Color::Rgb(180, 130, 30)),
        },
    }
}

fn effect_nerd_icon(effect: TileEffect) -> &'static str {
    match effect {
        TileEffect::Normal => " ",
        TileEffect::Crystal => icons::CRYSTAL,
        TileEffect::Poison => icons::POISON,
        TileEffect::Spike => icons::SPIKE,
        TileEffect::Plague => icons::PLAGUE,
        TileEffect::Stone => icons::STONE,
        TileEffect::Whirlwind => icons::WIND,
        TileEffect::Duplicator => icons::DUPLICATE,
        TileEffect::Flipped => icons::FLIP,
        TileEffect::Broken => icons::CRACK,
    }
}

fn tile_colors(rarity: TileRarity, effect: TileEffect) -> (Color, Color) {
    match effect {
        TileEffect::Normal => match rarity {
            TileRarity::Bronze => (Color::Rgb(240, 240, 250), Color::Rgb(70, 65, 90)),
            TileRarity::Silver => (Color::Rgb(240, 240, 255), Color::Rgb(55, 65, 140)),
            TileRarity::Gold => (Color::Rgb(255, 250, 200), Color::Rgb(155, 110, 25)),
        },
        TileEffect::Crystal => (Color::Rgb(220, 255, 255), Color::Rgb(25, 110, 130)),
        TileEffect::Poison => (Color::Rgb(220, 255, 220), Color::Rgb(30, 110, 35)),
        TileEffect::Spike => (Color::Rgb(255, 220, 220), Color::Rgb(150, 40, 40)),
        TileEffect::Plague => (Color::Rgb(255, 200, 255), Color::Rgb(130, 35, 130)),
        TileEffect::Stone => (Color::Rgb(120, 120, 120), Color::Rgb(55, 55, 55)),
        TileEffect::Whirlwind => (Color::Rgb(255, 235, 160), Color::Rgb(90, 65, 130)),
        TileEffect::Duplicator => (Color::Rgb(200, 240, 255), Color::Rgb(35, 75, 130)),
        TileEffect::Flipped => (Color::Rgb(250, 250, 210), Color::Rgb(110, 110, 55)),
        TileEffect::Broken => (Color::Rgb(170, 170, 170), Color::Rgb(75, 75, 75)),
    }
}

fn rarity_dot_color(rarity: TileRarity) -> Color {
    match rarity {
        TileRarity::Bronze => Color::Rgb(180, 140, 100),
        TileRarity::Silver => Color::Rgb(160, 180, 220),
        TileRarity::Gold => Color::Rgb(255, 215, 0),
    }
}

fn effect_indicator_color(effect: TileEffect) -> Color {
    match effect {
        TileEffect::Crystal => Color::Rgb(80, 255, 255),
        TileEffect::Poison => Color::Rgb(80, 255, 80),
        TileEffect::Spike => Color::Rgb(255, 80, 80),
        TileEffect::Plague => Color::Rgb(255, 80, 255),
        TileEffect::Stone => Color::Rgb(140, 140, 140),
        TileEffect::Whirlwind => Color::Rgb(255, 200, 80),
        TileEffect::Duplicator => Color::Rgb(80, 180, 255),
        TileEffect::Flipped => Color::Rgb(200, 200, 120),
        TileEffect::Broken => Color::Rgb(180, 80, 80),
        TileEffect::Normal => Color::White,
    }
}
