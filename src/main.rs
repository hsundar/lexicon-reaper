mod app;
mod dict;
mod event;
mod game;
mod generation;
mod save;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use dict::Dictionary;
use event::EventHandler;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Load dictionary
    eprintln!("Loading dictionary...");
    let dictionary = Dictionary::new();
    eprintln!("Loaded {} words.", dictionary.word_count());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event handler
    let mut app = App::new(dictionary);
    let events = EventHandler::new();

    // Main loop
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        let event = events.next()?;
        app.update(event);

        if app.should_quit {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
