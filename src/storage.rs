use crate::app::Chat;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const CHATS_FILE: &str = "chats.json";
#[cfg(target_os = "linux")]
const LINUX_DATA_DIR: &str = "lumenchat";

#[cfg(target_os = "linux")]
fn chats_path() -> PathBuf {
    let base_dirs = directories_next::BaseDirs::new();
    let xdg_data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());

    base_dirs
        .map(|dirs| chats_path_in_data_dir(&linux_data_dir(
            xdg_data_home.as_deref(),
            dirs.home_dir(),
        )))
        .unwrap_or_else(|| PathBuf::from(CHATS_FILE))
}

#[cfg(not(target_os = "linux"))]
fn chats_path() -> PathBuf {
    match std::env::current_exe() {
        Ok(exe) => chats_path_next_to(&exe),
        Err(_) => PathBuf::from(CHATS_FILE),
    }
}

#[cfg(target_os = "linux")]
fn chats_path_in_data_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(CHATS_FILE)
}

#[cfg(target_os = "linux")]
fn linux_data_dir(xdg_data_home: Option<&Path>, home_dir: &Path) -> PathBuf {
    xdg_data_home
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home_dir.join(".local").join("share"))
        .join(LINUX_DATA_DIR)
}

#[cfg(not(target_os = "linux"))]
fn chats_path_next_to(executable: &Path) -> PathBuf {
    executable
        .parent()
        .map(|parent| parent.join(CHATS_FILE))
        .unwrap_or_else(|| PathBuf::from(CHATS_FILE))
}

pub const fn chat_history_location_description() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Chat history is stored in your local XDG data directory."
    }

    #[cfg(not(target_os = "linux"))]
    {
        "Chat history is stored locally next to the executable."
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
    save_chats_at(&path, chats)
}

fn save_chats_at(path: &Path, chats: &[Chat]) -> Result<()> {
    let json = serde_json::to_vec_pretty(chats)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    // Evil atomic write (temp to rename)
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)
        .with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .with_context(|| format!("renaming to {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn executable_adjacent_path_uses_the_executables_parent_directory() {
        assert_eq!(
            chats_path_next_to(Path::new("/opt/lumen-chat/lumen-chat")),
            PathBuf::from("/opt/lumen-chat/chats.json")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_data_path_uses_the_provided_xdg_data_directory() {
        assert_eq!(
            chats_path_in_data_dir(Path::new("/home/alice/.local/share/lumen-chat")),
            PathBuf::from("/home/alice/.local/share/lumen-chat/chats.json")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_data_directory_respects_xdg_data_home() {
        assert_eq!(
            linux_data_dir(
                Some(Path::new("/var/lib/alice-data")),
                Path::new("/home/alice")
            ),
            PathBuf::from("/var/lib/alice-data/lumenchat")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_data_directory_defaults_to_local_share_when_xdg_is_unset() {
        assert_eq!(
            linux_data_dir(None, Path::new("/home/alice")),
            PathBuf::from("/home/alice/.local/share/lumenchat")
        );
    }

    #[test]
    fn save_creates_missing_parent_directories_and_round_trips() {
        let temp_root =
            std::env::temp_dir().join(format!("lumenchat-storage-test-{}", uuid::Uuid::new_v4()));
        let path = temp_root.join("nested").join(CHATS_FILE);
        let chats = vec![Chat::new("test-model".into())];

        save_chats_at(&path, &chats).expect("save chat history");
        let loaded: Vec<Chat> =
            serde_json::from_slice(&std::fs::read(&path).expect("read test history"))
                .expect("parse test chat history");

        assert_eq!(loaded.len(), 1);
        std::fs::remove_dir_all(temp_root).expect("remove test directory");
    }
}
