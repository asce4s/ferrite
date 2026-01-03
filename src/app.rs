use crate::auth::AuthError;
use crate::util::Session;
use crate::widgets::{select::SelectField, text::TextField};
use tui_input::Input;

#[derive(Debug)]
pub enum AuthState {
    None,
    Authenticating,
    Failed(AuthError),
}

#[derive(Debug)]
pub struct AppState {
    pub username: SelectField<String, fn(&String) -> String>,
    pub password: TextField,
    pub session: SelectField<Session, fn(&Session) -> String>,
    pub focus_index: u8,
    pub max_focus_index: u8,
    pub auth_state: AuthState,
    pub hostname: String,
}

impl AppState {
    pub fn new(sessions: Vec<Session>, users: Vec<String>, hostname: String) -> Self {
        Self {
            auth_state: AuthState::None,
            focus_index: 0,
            session: SelectField {
                selected_idx: 0,
                label: String::from("Session"),
                index: 0,
                items: sessions,
                transform: |s: &Session| s.name.clone(),
            },
            username: SelectField {
                index: 1,
                label: String::from("Username"),
                items: users,
                transform: |s: &String| s.clone(),
                selected_idx: 0,
            },
            password: TextField {
                index: 2,
                label: String::from("Password"),
                input: Input::default(),
                mask: Some(String::from("*")),
            },
            max_focus_index: 2,
            hostname,
        }
    }

    pub fn focus_next(&mut self) {
        let next_idx = self.focus_index + 1;
        if next_idx <= self.max_focus_index {
            self.focus_index = next_idx;
        }
    }

    pub fn focus_prev(&mut self) {
        self.focus_index = self.focus_index.saturating_sub(1);
    }
}
