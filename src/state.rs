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

fn state_path() -> PathBuf {
    PathBuf::from("/var/lib/ferrite/state.json")
}

pub fn load_state() -> FerriteState {
    let path = state_path();

    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_state(state: &FerriteState) -> anyhow::Result<()> {
    let path = state_path();

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
