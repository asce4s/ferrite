use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget, block::Position},
};
use crate::app::{AppState, AuthState};
use crate::auth::AuthError;
use crate::widgets::widget::InputField;

pub fn render(frame: &mut Frame, app_state: &mut AppState) {
    let fg_color = Color::White;
    let bg_color = Color::Black;

    let [header_area, content_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .margin(1)
    .areas(frame.area());

    frame.render_widget(
        Block::default().style(Style::default().bg(bg_color)),
        frame.area(),
    );

    Block::bordered()
        .border_type(BorderType::Plain)
        .fg(fg_color)
        .render(content_area, frame.buffer_mut());

    Block::bordered()
        .border_type(BorderType::Plain)
        .fg(fg_color)
        .render(header_area, frame.buffer_mut());

    let (title_txt, error_msg) = get_title_and_error(&app_state.auth_state);
    let footer_text = error_msg
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();

    let footer_color = match &error_msg {
        Some(_) => Color::Red,
        None => fg_color,
    };
    Paragraph::new(footer_text)
        .block(
            Block::bordered()
                .border_type(BorderType::Plain)
                .fg(footer_color),
        )
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
        .border_type(BorderType::Plain)
        .fg(fg_color)
        .title(title_txt)
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

fn get_title_and_error(auth_state: &AuthState) -> (&'static str, Option<String>) {
    match auth_state {
        AuthState::None => ("Authenticate", None),
        AuthState::Authenticating => ("Authenticating", None),
        AuthState::Failed(auth_error) => match auth_error {
            AuthError::AuthFailed(_) => ("Authentication Failed", None),
            AuthError::Connection(e) | AuthError::Protocol(e) | AuthError::InvalidSession(e) => {
                ("Authenticate", Some(e.to_string()))
            }
        },
    }
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

