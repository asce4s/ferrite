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
use crate::state::{FerriteState, load_state, save_state, state_path};
use crate::ui::render;
use crate::util::{get_login_users, read_sessions};
use crate::widgets::widget::InputField;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event as term_event;
use std::io::{Write, stdout};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    color_eyre::install()?;

    let sessions = read_sessions()?;
    let users = get_login_users("/etc/passwd")?;
    let state = load_state(state_path());
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let mut app_state = AppState::new(sessions, users, hostname, state);

    // Suppress kernel printk messages from reaching this TTY while the greeter
    // owns it. These are written directly by the kernel (e.g. TLP power events,
    // udev) and bypass ratatui's rendering entirely.
    let saved_printk = console_silence();

    // Full VT reset before ratatui takes over, clearing any boot/service output.
    print!("\x1bc");
    let _ = stdout().flush();

    let terminal = ratatui::init();
    let result = run(terminal, &mut app_state);

    ratatui::restore();
    console_restore(saved_printk);
    result
}

/// Suppress kernel messages on the console TTY (equivalent to `dmesg -n 0`).
/// Returns the original value to restore on exit.
fn console_silence() -> Option<String> {
    let original = std::fs::read_to_string("/proc/sys/kernel/printk").ok()?;
    std::fs::write("/proc/sys/kernel/printk", "0 4 1 7\n").ok()?;
    Some(original)
}

fn console_restore(saved: Option<String>) {
    if let Some(value) = saved {
        let _ = std::fs::write("/proc/sys/kernel/printk", value);
    }
}

// During boot, some kernel events arrive late. Force full repaints so ratatui's
// differential rendering doesn't leave corrupted cells unrepaired.
const BOOT_REPAINT_WINDOW: Duration = Duration::from_secs(5);
const BOOT_POLL_INTERVAL: Duration = Duration::from_millis(100);

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    let boot_deadline = Instant::now() + BOOT_REPAINT_WINDOW;

    loop {
        if Instant::now() < boot_deadline {
            // Force a full repaint: marks every cell dirty so ratatui redraws
            // everything, not just cells it thinks changed.
            terminal.clear()?;
        }
        terminal.draw(|f| render(f, app_state))?;

        let evt = if Instant::now() < boot_deadline {
            if term_event::poll(BOOT_POLL_INTERVAL)? {
                term_event::read()?
            } else {
                continue;
            }
        } else {
            term_event::read()?
        };

        match handle_event(&evt, app_state)? {
            Action::Quit => return Ok(()),
            Action::Continue => continue,
            Action::Authenticate => {
                app_state.auth_state = AuthState::Authenticating;
                terminal.draw(|f| render(f, app_state))?;

                let res = authenticate(
                    &app_state.username.get_value(),
                    &app_state.password.get_value(),
                    &app_state.session.get_value().exec,
                );

                match res {
                    Ok(_) => {
                        let state = FerriteState {
                            last_user: Some(app_state.username.get_value()),
                            last_session: Some(app_state.session.get_value().name),
                            version: 1,
                        };
                        if let Err(e) = save_state(&state, state_path()) {
                            eprintln!("Warning: Failed to save state to {:?}: {e}", state_path());
                        }
                        return Ok(());
                    }
                    Err(err) => {
                        app_state.auth_state = AuthState::Failed(err);
                        app_state.password.input = tui_input::Input::default();
                    }
                }
            }
        }
    }
}
