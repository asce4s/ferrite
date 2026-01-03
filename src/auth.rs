use core::fmt;
use std::{env, os::unix::net::UnixStream};

use color_eyre::Result;
use greetd_ipc::codec::SyncCodec;
use greetd_ipc::{AuthMessageType, Request, Response};

#[derive(Debug)]
pub enum AuthError {
    AuthFailed(String),
    Connection(String),
    Protocol(String),
    InvalidSession(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::AuthFailed(msg) => write!(f, "Authentication failed: {msg}"),
            AuthError::Connection(msg) => write!(f, "Connection error: {msg}"),
            AuthError::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            AuthError::InvalidSession(msg) => write!(f, "Invalid session: {msg}"),
        }
    }
}

pub fn authenticate(
    username: &str,
    password: &str,
    session_cmd: &[String],
) -> Result<(), AuthError> {
    let socket_path = env::var("GREETD_SOCK").unwrap_or_else(|_| "/run/greetd.sock".to_string());

    let mut stream =
        UnixStream::connect(&socket_path).map_err(|e| AuthError::Connection(e.to_string()))?;

    Request::CreateSession {
        username: username.to_string(),
    }
    .write_to(&mut stream)
    .map_err(|e| AuthError::Protocol(e.to_string()))?;

    loop {
        match Response::read_from(&mut stream).map_err(|e| AuthError::Protocol(e.to_string()))? {
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
                    .map_err(|e| AuthError::Protocol(e.to_string()))?;
            }
            Response::Success => break,
            Response::Error { description, .. } => {
                let _ = Request::CancelSession.write_to(&mut stream);
                return Err(AuthError::AuthFailed(description));
            }
        }
    }

    if session_cmd.is_empty() {
        return Err(AuthError::InvalidSession(
            "no session command provided".into(),
        ));
    }

    Request::StartSession {
        cmd: session_cmd.to_vec(),
        env: Vec::new(),
    }
    .write_to(&mut stream)
    .map_err(|e| AuthError::Protocol(e.to_string()))?;

    match Response::read_from(&mut stream).map_err(|e| AuthError::Protocol(e.to_string()))? {
        Response::Success => Ok(()),
        Response::AuthMessage { .. } => Err(AuthError::InvalidSession(
            "unexpected auth prompt after start_session".into(),
        )),
        Response::Error { description, .. } => Err(AuthError::Protocol(description)),
    }
}
