use crate::power::{PowerAction, power};
use crate::widgets::widget::InputField;
use crate::app::AppState;
use ratatui::crossterm::event::{Event, KeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
    Authenticate,
}
pub fn handle_event(event: &Event, app_state: &mut AppState) -> Result<Action, color_eyre::Report> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Esc => return Ok(Action::Quit),
            KeyCode::Down => app_state.focus_next(),
            KeyCode::Up => app_state.focus_prev(),
            KeyCode::Enter => return Ok(Action::Authenticate),
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
