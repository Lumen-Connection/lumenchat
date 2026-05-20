use crate::app::Chat;
use anyhow::{Context, Result};
use std::path::PathBuf;

const CHATS_FILE: &str = "chats.json";

fn chats_path() -> PathBuf {
    match std::env::current_exe() {
        Ok(exe) => exe
            .parent()
            .map(|p| p.join(CHATS_FILE))
            .unwrap_or_else(|| PathBuf::from(CHATS_FILE)),
        Err(_) => PathBuf::from(CHATS_FILE),
    }
}

pub fn load_chats() -> Result<Vec<Chat>> {
    let path = chats_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = std::fs::read(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let chats: Vec<Chat> = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(chats)
}

pub fn save_chats(chats: &[Chat]) -> Result<()> {
    let path = chats_path();
    let json = serde_json::to_vec_pretty(chats)?;
    // Evil ass atomic "write" (temp to rename)
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)
        .with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .with_context(|| format!("renaming to {}", path.display()))?;
    Ok(())
}