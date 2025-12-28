mod auth;
mod util;
mod widgets;

use color_eyre::{Result, eyre::Ok};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget, block::Position},
};
use tui_input::Input;

use crate::{
    auth::authenticate,
    util::{Session, get_login_users, read_sessions},
    widgets::{select::SelectField, text::TextField, widget::InputField},
};

#[derive(Debug)]
struct AppState {
    username: SelectField<String, fn(&String) -> String>,
    password: TextField,
    session: SelectField<Session, fn(&Session) -> String>,
    focus_index: u8,
    max_focus_index: u8,
    error_msg: Option<String>,
}

impl AppState {
    fn focus_next(&mut self) {
        let next_idx = self.focus_index + 1;
        if next_idx <= self.max_focus_index {
            self.focus_index = next_idx;
        }
    }
    fn focus_prev(&mut self) {
        self.focus_index = self.focus_index.saturating_sub(1);
    }
}
fn main() -> Result<()> {
    let sessions = read_sessions().unwrap();
    let users = get_login_users().unwrap();

    let mut state = AppState {
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
        max_focus_index: 3,
        error_msg: None,
    };

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &mut state);
    ratatui::restore();
    result
    //
    // Ok(())
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app_state))?;
        let event = event::read()?;

        if let Event::Key(key) = &event {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Down => app_state.focus_next(),
                KeyCode::Up => app_state.focus_prev(),
                KeyCode::Enter => {
                    app_state.error_msg = None;
                    let res = authenticate(
                        &app_state.username.get_value(),
                        &app_state.password.get_value(),
                        &[app_state.session.get_value().exec],
                    );
                    if let Err(err) = res {
                        app_state.error_msg = Some(err.to_string())
                    }
                }
                _ => {}
            }
        };
        app_state
            .username
            .handle_event(app_state.focus_index, &event);

        app_state
            .password
            .handle_event(app_state.focus_index, &event);

        app_state
            .session
            .handle_event(app_state.focus_index, &event);
    }
    // Ok(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let [header_area, content_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .margin(1)
    .areas(frame.area());

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Blue)
        .render(content_area, frame.buffer_mut());

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Blue)
        .render(header_area, frame.buffer_mut());

    let footer_text = app_state
        .error_msg
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();

    let color = match app_state.error_msg.as_ref() {
        Some(_) => Color::Red,
        None => Color::Blue,
    };

    Paragraph::new(footer_text)
        .block(Block::bordered().border_type(BorderType::Rounded).fg(color))
        .render(footer_area, frame.buffer_mut());

    let main_block = centered_rect(40, 11, content_area);

    let [session_area, username_area, password_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .margin(1)
    .areas(main_block);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Blue)
        .title("Authenticate")
        .title_position(Position::Top)
        .title_alignment(Alignment::Center)
        .render(main_block, frame.buffer_mut());

    app_state
        .username
        .render(frame, &app_state.focus_index, username_area);
    app_state
        .password
        .render(frame, &app_state.focus_index, password_area);
    app_state
        .session
        .render(frame, &app_state.focus_index, session_area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    // Calculate vertical centering
    let vertical_margin = area.height.saturating_sub(height) / 2;
    let vertical_layout = Layout::vertical([
        Constraint::Length(vertical_margin),
        Constraint::Length(height),
        Constraint::Length(vertical_margin),
    ])
    .split(area);

    // Calculate horizontal centering
    let horizontal_margin = area.width.saturating_sub(width) / 2;
    let horizontal_layout = Layout::horizontal([
        Constraint::Length(horizontal_margin),
        Constraint::Length(width),
        Constraint::Length(horizontal_margin),
    ])
    .split(vertical_layout[1]);

    horizontal_layout[1]
}
