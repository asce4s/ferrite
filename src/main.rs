mod auth;
mod util;
mod widgets;

use std::{
    any,
    fs::{self, File},
    io::{self, BufRead},
};

use anyhow::Context;
use color_eyre::{Result, eyre::Ok};

use ini::configparser::ini::Ini;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Widget, block::Position},
};
use tui_input::Input;

use crate::{
    auth::authenticate,
    widgets::{select::SelectField, text::TextField, widget::InputField},
};

#[derive(Debug)]
struct AppState {
    username: SelectField<String, fn(&String) -> String>,
    password: TextField,
    session: SelectField<Session, fn(&Session) -> String>,
    focus_index: u8,
    max_focus_index: u8,
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

#[derive(Debug, Default, Clone)]
struct Session {
    name: String,
    exec: String,
}

fn read_sessions() -> anyhow::Result<Vec<Session>> {
    anyhow::Ok(
        fs::read_dir("/usr/share/wayland-sessions")?
            .filter_map(|e| {
                let path = e.ok()?.path();
                (path.extension()?.to_str()? == "desktop").then_some(path)
            })
            .filter_map(|path| {
                let mut ini = Ini::new();
                let conf = ini.load(path.to_str()?).ok()?;
                let s = conf.get("desktop entry")?;

                Some(Session {
                    name: s.get("name")?.clone()?,
                    exec: s.get("exec")?.clone()?,
                })
            })
            .collect::<Vec<Session>>(),
    )
}
fn get_login_users() -> anyhow::Result<Vec<String>> {
    let file = File::open("/etc/passwd")?;
    let reader = io::BufReader::new(file);

    let mut users = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue; // skip malformed lines
        }

        let username = fields[0];
        let uid: u32 = fields[2].parse().unwrap_or(1);
        let shell = fields[6];

        if (uid == 0 || uid >= 1000) && !shell.ends_with("nologin") && shell != "/bin/false" {
            users.push(username.to_string());
        }
    }

    anyhow::Ok(users)
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

        app_state
            .username
            .handle_event(app_state.focus_index, &event);

        app_state
            .password
            .handle_event(app_state.focus_index, &event);

        app_state
            .session
            .handle_event(app_state.focus_index, &event);

        if let Event::Key(key) = &event {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Down => app_state.focus_next(),
                KeyCode::Up => app_state.focus_prev(),
                KeyCode::Enter => {
                    let res = authenticate(
                        &app_state.username.get_value(),
                        &app_state.password.get_value(),
                        &[app_state.session.get_value().exec],
                    );
                }
                _ => {}
            }
        };
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
        .fg(Color::Red)
        .render(content_area, frame.buffer_mut());

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Yellow)
        .render(header_area, frame.buffer_mut());

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(Color::Blue)
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
