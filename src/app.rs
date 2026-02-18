use crate::auth::AuthError;
use crate::state::FerriteState;
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
    pub fn new(
        sessions: Vec<Session>,
        users: Vec<String>,
        hostname: String,
        state: FerriteState,
    ) -> Self {
        let last_session = state
            .last_session
            .as_deref()
            .and_then(|name| sessions.iter().position(|s| s.name == name));

        let last_user = state
            .last_user
            .as_deref()
            .and_then(|user| users.iter().position(|u| u == user));

        let focus_index = (last_session.is_some() && last_user.is_some()) as u8 * 2;

        Self {
            auth_state: AuthState::None,
            focus_index,
            session: SelectField {
                selected_idx: last_session.unwrap_or(0),
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
                selected_idx: last_user.unwrap_or(0),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let sessions = vec![
            Session {
                name: "Wayland".to_string(),
                exec: "wayland".to_string(),
            },
            Session {
                name: "X11".to_string(),
                exec: "x11".to_string(),
            },
        ];
        let users = vec!["alice".to_string(), "bob".to_string()];
        let hostname = "ferrite-host".to_string();
        let state = FerriteState {
            version: 1,
            last_user: Some("bob".to_string()),
            last_session: Some("X11".to_string()),
        };

        let app_state = AppState::new(sessions, users, hostname, state);

        assert_eq!(app_state.hostname, "ferrite-host");
        assert_eq!(app_state.username.selected_idx, 1); // bob
        assert_eq!(app_state.session.selected_idx, 1); // X11
        assert_eq!(app_state.focus_index, 2); // password field
    }

    #[test]
    fn test_app_state_focus_navigation() {
        let sessions = vec![Session {
            name: "Wayland".to_string(),
            exec: "wayland".to_string(),
        }];
        let users = vec!["alice".to_string()];
        let hostname = "ferrite-host".to_string();
        let state = FerriteState::default();

        let mut app_state = AppState::new(sessions, users, hostname, state);

        assert_eq!(app_state.focus_index, 0);

        app_state.focus_next();
        assert_eq!(app_state.focus_index, 1);

        app_state.focus_next();
        assert_eq!(app_state.focus_index, 2);

        // Boundary
        app_state.focus_next();
        assert_eq!(app_state.focus_index, 2);

        app_state.focus_prev();
        assert_eq!(app_state.focus_index, 1);

        app_state.focus_prev();
        assert_eq!(app_state.focus_index, 0);

        // Boundary
        app_state.focus_prev();
        assert_eq!(app_state.focus_index, 0);
    }
}
