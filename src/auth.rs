use std::{env, os::unix::net::UnixStream};

use color_eyre::{Result, eyre::eyre};
use greetd_ipc::codec::SyncCodec;
use greetd_ipc::{AuthMessageType, Request, Response};

pub fn authenticate(username: &str, password: &str, session_cmd: &[String]) -> Result<()> {
    let socket_path = env::var("GREETD_SOCK").unwrap_or_else(|_| "/run/greetd.sock".to_string());

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| eyre!("failed to connect to greetd socket {}: {}", socket_path, e))?;

    Request::CreateSession {
        username: username.to_string(),
    }
    .write_to(&mut stream)
    .map_err(|e| eyre!("failed to send create_session: {}", e))?;

    loop {
        match Response::read_from(&mut stream)
            .map_err(|e| eyre!("failed to read auth response: {}", e))?
        {
            Response::AuthMessage {
                auth_message_type, ..
            } => {
                let response = match auth_message_type {
                    AuthMessageType::Visible | AuthMessageType::Secret => {
                        Some(password.to_string())
                    }
                    _ => None,
                };

                Request::PostAuthMessageResponse { response }
                    .write_to(&mut stream)
                    .map_err(|e| eyre!("failed to send auth response: {}", e))?;
            }
            Response::Success => break,
            Response::Error {
                error_type,
                description,
            } => {
                return Err(eyre!(
                    "greetd auth failed ({:?}): {}",
                    error_type,
                    description
                ));
            }
        }
    }

    if session_cmd.is_empty() {
        return Err(eyre!("no session command provided"));
    }

    Request::StartSession {
        cmd: session_cmd.to_vec(),
        env: Vec::new(),
    }
    .write_to(&mut stream)
    .map_err(|e| eyre!("failed to send start_session: {}", e))?;

    match Response::read_from(&mut stream)
        .map_err(|e| eyre!("failed to read start_session response: {}", e))?
    {
        Response::Success => Ok(()),
        Response::AuthMessage { .. } => Err(eyre!("unexpected auth prompt after start_session")),
        Response::Error {
            error_type,
            description,
        } => Err(eyre!(
            "start_session failed ({:?}): {}",
            error_type,
            description
        )),
    }
}
