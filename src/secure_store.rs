use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "openchat";
const USER: &str = "openrouter_api_key";

pub struct SecureStore;

impl SecureStore {
    fn entry() -> Result<Entry> {
        Entry::new(SERVICE, USER).context("Failed to open WCM entry")
    }

    pub fn load_key() -> Result<Option<String>> {
        match Self::entry()?.get_password() {
            Ok(secret) => {
                let trimmed = secret.trim().to_string();
                if trimmed.is_empty() {
                    Ok(None)
                }
                else {
                    Ok(Some(trimmed))
                }
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e).context("Failed to read API from store"),
        }
    }

    pub fn save_key(key: &str) -> Result<()> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            anyhow::bail!("API key is empty.");
        }
        Self::entry()?
            .set_password(trimmed)
            .context("Failed to write to store")
    }

    #[allow(dead_code)]
    pub fn clear_key() -> Result<()> {
        match Self::entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e).context("Failed to delete key from store"),
        }
    }
}