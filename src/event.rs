use crate::power::{PowerAction, power};
use crate::state::{FerriteState, save_state};
use crate::widgets::widget::InputField;
use crate::{app::AppState, auth::authenticate};
use ratatui::crossterm::event::{Event, KeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
}
pub fn handle_event(event: &Event, app_state: &mut AppState) -> Result<Action, color_eyre::Report> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Esc => return Ok(Action::Quit),
            KeyCode::Down => app_state.focus_next(),
            KeyCode::Up => app_state.focus_prev(),
            KeyCode::Enter => {
                app_state.auth_state = crate::app::AuthState::Authenticating;
                let res = authenticate(
                    &app_state.username.get_value(),
                    &app_state.password.get_value(),
                    &[app_state.session.get_value().exec],
                );

                match res {
                    Ok(_) => {
                        let state = FerriteState {
                            last_user: Some(app_state.username.get_value()),
                            last_session: Some(app_state.session.get_value().name),
                            version: 1,
                        };
                        let _=save_state(&state) ;// handle later
                        return Ok(Action::Quit);
                    }
                    Err(err) => {
                        app_state.auth_state = crate::app::AuthState::Failed(err);
                    }
                }
            }
            KeyCode::F(1) => power(PowerAction::Shutdown),
            KeyCode::F(2) => power(PowerAction::Reboot),

            _ => {
                app_state
                    .username
                    .handle_event(app_state.focus_index, event);

                app_state
                    .password
                    .handle_event(app_state.focus_index, event);

                app_state.session.handle_event(app_state.focus_index, event);
            }
        }
    }
    Ok(Action::Continue)
}
