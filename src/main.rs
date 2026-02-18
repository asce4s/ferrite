mod app;
mod auth;
mod event;
mod power;
mod state;
mod ui;
mod util;
mod widgets;

use crate::app::{AppState, AuthState};
use crate::auth::authenticate;
use crate::event::{Action, handle_event};
use crate::state::{FerriteState, load_state, save_state};
use crate::ui::render;
use crate::util::{get_login_users, read_sessions};
use crate::widgets::widget::InputField;
use color_eyre::Result;
use ratatui::DefaultTerminal;

fn main() -> Result<()> {
    color_eyre::install()?;

    let sessions = read_sessions()?;
    let users = get_login_users()?;
    let state = load_state();
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let mut app_state = AppState::new(sessions, users, hostname, state);

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
            Action::Authenticate => {
                app_state.auth_state = AuthState::Authenticating;
                terminal.draw(|f| render(f, app_state))?;

                let username = app_state.username.get_value().unwrap_or_default();
                let password = app_state.password.get_value().unwrap_or_default();
                let session = app_state.session.get_value();

                if session.is_none() {
                    app_state.auth_state = AuthState::Failed(crate::auth::AuthError::InvalidSession(
                        "No session selected".into(),
                    ));
                    continue;
                }
                let session = session.unwrap();

                match authenticate(&username, &password, &session.exec) {
                    Ok(_) => {
                        let state = FerriteState {
                            last_user: Some(username),
                            last_session: Some(session.name),
                            version: 1,
                        };
                        if let Err(e) = save_state(&state) {
                            eprintln!("Failed to save state: {e}");
                        }
                        return Ok(());
                    }
                    Err(err) => {
                        app_state.auth_state = AuthState::Failed(err);
                        app_state.clear_password();
                    }
                }
            }
        }
    }
}
