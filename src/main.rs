mod app;
mod auth;
mod event;
mod power;
mod state;
mod ui;
mod util;
mod widgets;

use crate::app::AppState;
use crate::event::{Action, handle_event};
use crate::state::load_state;
use crate::ui::render;
use crate::util::{get_login_users, read_sessions};
use color_eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::{
    execute,
    terminal::EnterAlternateScreen,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let sessions = read_sessions()?;
    let users = get_login_users()?;
    let state = load_state();
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let mut app_state = AppState::new(sessions, users, hostname, state);

    // Explicitly enter alternate screen mode for fullscreen
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let terminal = ratatui::init();
    let result = run(terminal, &mut app_state);
    
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app_state))?;
        let evt = ratatui::crossterm::event::read()?;

        match handle_event(&evt, app_state)? {
            Action::Quit => return Ok(()),
            Action::Continue => continue,
        }
    }
}
