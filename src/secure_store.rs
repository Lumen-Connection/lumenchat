use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "openchat";
const USER: &str = "openrouter_api_key";

pub struct SecureStore;

impl SecureStore {
    pub const fn display_name() -> &'static str {
        #[cfg(windows)]
        {
            "Windows Credential Manager"
        }

        #[cfg(target_os = "linux")]
        {
            "your Linux Secret Service (such as GNOME Keyring or KWallet)"
        }
    }

    fn entry() -> Result<Entry> {
        Entry::new(SERVICE, USER).context("Failed to open system credential-store entry")
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
            Err(e) => Err(e).context("Failed to read API key from system credential store"),
        }
    }

    pub fn save_key(key: &str) -> Result<()> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            anyhow::bail!("API key is empty.");
        }
        Self::entry()?
            .set_password(trimmed)
            .context("Failed to write API key to system credential store")
    }

    pub fn delete_key() -> anyhow::Result<()> {
        let entry = keyring::Entry::new(SERVICE, USER)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
