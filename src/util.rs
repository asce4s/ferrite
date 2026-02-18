use std::{
    env,
    fs::{self, File},
    io::{self, BufRead},
};
use color_eyre::Result;

#[derive(Debug, Default, Clone)]
pub struct Session {
    pub name: String,
    pub exec: String,
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
            if let Some((key, value)) = line.split_once("=") {
                match key.trim() {
                    "Name" => session.name = value.to_string(),
                    "Exec" => session.exec = value.to_string(),
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

// read_sessions_in_dir(dir: &str) -> anyhow::Result<Vec<Session>> {
//     let sessions = fs::read_dir(dir)?
//         .filter_map(|e| {
//             let path = e.ok()?.path();
//             (path.extension()?.to_str()? == "desktop").then_some(path)
//         })
//         .filter_map(|path| {
//             let file = File::open(path).expect("Unable to read file");
//             let reader = io::BufReader::new(file);
//
//             let mut session = Session::default();
//
//             for line in reader.lines() {
//                 let parts: Vec<&str> = line.unwrap().split("=").collect();
//
//                 if !session.name.is_empty() && !session.exec.is_empty() {
//                     break;
//                 }
//
//                 let key = parts.get(0).unwrap_or(&"").to_string();
//                 let value = parts.get(1).unwrap_or(&"").to_string();
//
//                 if key.trim() == "Name" {
//                     session.name = value.clone();
//                 }
//
//                 if key.trim() == "Exec" {
//                     session.exec = value.clone()
//                 }
//             }
//
//             if session.name.is_empty() || session.exec.is_empty() {
//                 return None;
//             }
//
//             Some(session)
//         })
//         .collect::<Vec<Session>>();
//
//     Ok(sessions)
// }
//
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

    let mut users = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue; // skip malformed lines
        }

        let username = fields[0];
        let uid: u32 = match fields[2].parse() {
            Ok(uid) => uid,
            Err(_) => continue,
        };
        let shell = fields[6];

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
        assert_eq!(sessions[0].exec, "test-session");
    }

    #[test]
    fn test_get_login_users() {
        let dir = tempdir().unwrap();
        let passwd_path = dir.path().join("passwd");
        let mut file = File::create(&passwd_path).unwrap();
        writeln!(file, "root:x:0:0:root:/root:/bin/bash").unwrap();
        writeln!(file, "alice:x:1000:1000:Alice:/home/alice:/bin/bash").unwrap();
        writeln!(file, "bob:x:1001:1001:Bob:/home/bob:/usr/bin/zsh").unwrap();
        writeln!(file, "guest:x:1002:1002:Guest:/home/guest:/sbin/nologin").unwrap();
        writeln!(file, "nobody:x:65534:65534:nobody:/:/bin/false").unwrap();

        let users = get_login_users(passwd_path.to_str().unwrap()).unwrap();
        assert_eq!(users.len(), 3);
        assert!(users.contains(&"root".to_string()));
        assert!(users.contains(&"alice".to_string()));
        assert!(users.contains(&"bob".to_string()));
        assert!(!users.contains(&"guest".to_string()));
        assert!(!users.contains(&"nobody".to_string()));
    }
}
