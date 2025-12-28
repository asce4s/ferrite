use std::{
    env,
    fs::{self, File},
    io::{self, BufRead},
};

use ini::configparser::ini::Ini;

#[derive(Debug, Default, Clone)]
pub struct Session {
    pub name: String,
    pub exec: String,
}

fn read_sessions_in_dir(dir: &str) -> anyhow::Result<Vec<Session>> {
    let sessions = fs::read_dir(dir)?
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
        .collect::<Vec<Session>>();

    Ok(sessions)
}

pub fn read_sessions() -> anyhow::Result<Vec<Session>> {
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
pub fn get_login_users() -> anyhow::Result<Vec<String>> {
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
