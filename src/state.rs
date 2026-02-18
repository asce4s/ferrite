use anyhow::Ok;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FerriteState {
    pub version: u8,
    pub last_user: Option<String>,
    pub last_session: Option<String>,
}

pub fn state_path() -> PathBuf {
    PathBuf::from("/var/lib/ferrite/state.json")
}

pub fn load_state(path: PathBuf) -> FerriteState {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_state(state: &FerriteState, path: PathBuf) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?
    }

    let json = serde_json::to_string(state)?;

    let mut tmp = NamedTempFile::new_in(path.parent().unwrap())?;
    tmp.write_all(json.as_bytes())?;
    tmp.flush()?;
    tmp.persist(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_state() {
        let dir = tempdir().unwrap();
        let state_file = dir.path().join("state.json");

        let state = FerriteState {
            version: 2,
            last_user: Some("alice".to_string()),
            last_session: Some("gnome".to_string()),
        };

        save_state(&state, state_file.clone()).unwrap();

        let loaded = load_state(state_file);

        assert_eq!(loaded.version, 2);
        assert_eq!(loaded.last_user, Some("alice".to_string()));
        assert_eq!(loaded.last_session, Some("gnome".to_string()));
    }

    #[test]
    fn test_load_nonexistent_state() {
        let dir = tempdir().unwrap();
        let state_file = dir.path().join("nonexistent.json");

        let loaded = load_state(state_file);

        assert_eq!(loaded.version, 0);
        assert_eq!(loaded.last_user, None);
        assert_eq!(loaded.last_session, None);
    }
}
