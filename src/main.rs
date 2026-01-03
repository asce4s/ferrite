mod app;
mod auth;
mod event;
mod ui;
mod util;
mod widgets;

use color_eyre::Result;
use ratatui::DefaultTerminal;
use crate::app::AppState;
use crate::event::{handle_event, Action};
use crate::ui::render;
use crate::util::{get_login_users, read_sessions};

fn main() -> Result<()> {
    color_eyre::install()?;

    let sessions = read_sessions()?;
    let users = get_login_users()?;
    let mut app_state = AppState::new(sessions, users);

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
