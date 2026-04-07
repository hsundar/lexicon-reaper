pub mod layout;
pub mod tile_grid;
pub mod arena;
pub mod enemy_panel;
pub mod player_panel;
pub mod word_bar;
pub mod message_log;
pub mod title_screen;
pub mod shop;
pub mod inventory;
pub mod map_screen;
pub mod theme;
pub mod icons;

use ratatui::Frame;

use crate::app::App;
use crate::game::state::Screen;

pub fn render(frame: &mut Frame, app: &App) {
    match app.current_screen() {
        Screen::Title => title_screen::render(frame, app),
        Screen::Combat => layout::render_combat(frame, app),
        Screen::Map => map_screen::render(frame, app),
        Screen::Shop => shop::render(frame, app),
        Screen::Inventory => inventory::render(frame, app),
        Screen::GameOver => title_screen::render_game_over(frame, app),
    }
}
