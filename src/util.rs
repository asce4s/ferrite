use std::{
    env,
    fs::{self, File},
    io::{self, BufRead},
};
use color_eyre::Result;

#[derive(Debug, Default, Clone)]
pub struct Session {
    pub name: String,
    pub exec: Vec<String>,
}

fn split_shell_command(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for c in cmd.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        args.push(current);
    }

    // Filter out field codes like %u, %f, etc.
    args.into_iter()
        .filter(|arg| !arg.starts_with('%'))
        .collect()
}

fn read_sessions_in_dir(dir: &str) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(sessions),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => return Ok(sessions),
        };

        let path = entry.path();

        if path.extension().and_then(|v| v.to_str()) != Some("desktop") {
            continue;
        }

        let file = match File::open(path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let reader = io::BufReader::new(file);
        let mut session = Session::default();
        for line in reader.lines().map_while(Result::ok) {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => session.name = value.trim().to_string(),
                    "Exec" => session.exec = split_shell_command(value.trim()),
                    _ => {}
                }
            }
            if !session.name.is_empty() && !session.exec.is_empty() {
                break;
            }
        }

        if !session.name.is_empty() && !session.exec.is_empty() {
            sessions.push(session);
        }
    }

    Ok(sessions)
}

pub fn read_sessions() -> Result<Vec<Session>> {
    let mut sessions = Vec::new();

    let mut search_paths: Vec<String> = vec![
        "/usr/share/wayland-sessions".into(),
        "/usr/share/xsessions".into(),
        "/usr/local/share/wayland-sessions".into(),
        "/usr/local/share/xsessions".into(),
        "/etc/X11/Sessions".into(),
    ];
    if let Ok(home) = env::var("HOME") {
        search_paths.push(format!("{home}/.local/share/wayland-sessions"));
        search_paths.push(format!("{home}/.local/share/xsessions"));
    }

    for path in search_paths {
        if let Ok(mut found) = read_sessions_in_dir(&path) {
            sessions.append(&mut found);
        }
    }

    Ok(sessions)
}
pub fn get_login_users(path: &str) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    parse_passwd(reader)
}

fn parse_passwd<R: BufRead>(reader: R) -> Result<Vec<String>> {
    let mut users = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let mut fields = line.split(':');

        let username = match fields.next() {
            Some(u) => u,
            None => continue,
        };
        let _password = fields.next();
        let uid: u32 = fields.next().and_then(|u| u.parse().ok()).unwrap_or(1);
        let _gid = fields.next();
        let _gecos = fields.next();
        let _home = fields.next();
        let shell = match fields.next() {
            Some(s) => s,
            None => continue,
        };

        if (uid == 0 || uid >= 1000) && !shell.ends_with("nologin") && shell != "/bin/false" {
            users.push(username.to_string());
        }
    }

    Ok(users)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_split_shell_command() {
        assert_eq!(split_shell_command("ls -l"), vec!["ls", "-l"]);
        assert_eq!(
            split_shell_command("gnome-session --session=gnome"),
            vec!["gnome-session", "--session=gnome"]
        );
        assert_eq!(
            split_shell_command("command \"with spaces\""),
            vec!["command", "with spaces"]
        );
        assert_eq!(
            split_shell_command("command %u %f"),
            vec!["command"]
        );
        assert_eq!(
            split_shell_command("quoted\\ space"),
            vec!["quoted space"]
        );
    }

    #[test]
    fn test_read_sessions_in_dir() {
        let dir = tempdir().unwrap();
        let session_file_path = dir.path().join("test.desktop");
        let mut file = File::create(session_file_path).unwrap();
        writeln!(file, "[Desktop Entry]").unwrap();
        writeln!(file, "Name=TestSession").unwrap();
        writeln!(file, "Exec=test-session").unwrap();

        let sessions = read_sessions_in_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "TestSession");
        assert_eq!(sessions[0].exec, vec!["test-session"]);
    }

    #[test]
    fn test_parse_passwd() {
        let data = "root:x:0:0:root:/root:/bin/bash\n\
                    daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n\
                    bin:x:2:2:bin:/bin:/usr/sbin/nologin\n\
                    user:x:1000:1000:user:/home/user:/bin/zsh\n\
                    guest:x:1001:1001:guest:/home/guest:/bin/false";
        let reader = io::Cursor::new(data);
        let users = parse_passwd(reader).unwrap();
        assert_eq!(users, vec!["root", "user"]);
    }
}
