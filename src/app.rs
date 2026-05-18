use crate::openrouter::{OpenRouterClient, OpenRouterError};
use crate::secure_store::SecureStore;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use tokio::runtime::Runtime;

pub enum Screen {
    Onboarding(OnboardingState),
    Main(MainState),
}

pub struct OnboardingState {
    pub key_input: String,
    pub show_key: bool,
    pub status: OnboardingStatus,
    pub rx: Option<Receiver<ValidationResult>>,
}

#[derive(Default, Clone)]
pub enum OnboardingStatus {
    #[default]
    Idle,
    Validating,
    Error(String),
}

pub enum ValidationResult {
    Ok(String), // the validated key
    Err(String),
}

pub struct MainState {
    pub api_key: String,
    // The chat UI will hang off of this in the next step.
}

pub struct App {
    pub screen: Screen,
    pub rt: Arc<Runtime>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let rt = Arc::new(Runtime::new()?);

        // If we already have a cached key, skip onboarding entirely.
        let screen = match SecureStore::load_key()? {
            Some(key) => Screen::Main(MainState { api_key: key }),
            None => Screen::Onboarding(OnboardingState {
                key_input: String::new(),
                show_key: false,
                status: OnboardingStatus::Idle,
                rx: None,
            }),
        };

        Ok(Self { screen, rt })
    }

    /// Kick off async validation of the entered key.
    pub fn start_validation(&mut self) {
        let Screen::Onboarding(state) = &mut self.screen else {
            return;
        };
        let key = state.key_input.trim().to_string();
        if key.is_empty() {
            state.status = OnboardingStatus::Error("Please enter an API key.".into());
            return;
        }

        let (tx, rx): (Sender<ValidationResult>, Receiver<ValidationResult>) = mpsc::channel();
        state.rx = Some(rx);
        state.status = OnboardingStatus::Validating;

        let rt = self.rt.clone();
        rt.spawn(async move {
            let result = match OpenRouterClient::new(key.clone()) {
                Ok(client) => match client.validate_key().await {
                    Ok(_info) => ValidationResult::Ok(key),
                    Err(OpenRouterError::Unauthorized) => ValidationResult::Err(
                        "That key was rejected. Double-check it on openrouter.ai/keys.".into(),
                    ),
                    Err(OpenRouterError::Network(e)) => {
                        ValidationResult::Err(format!("Network error: {e}"))
                    }
                    Err(OpenRouterError::Unexpected { status, body }) => ValidationResult::Err(
                        format!("Unexpected response from OpenRouter (HTTP {status}): {body}"),
                    ),
                },
                Err(e) => ValidationResult::Err(format!("Client init failed: {e}")),
            };
            let _ = tx.send(result);
        });
    }

    /// Drain any pending validation result and react to it. Called every frame.
    pub fn poll_validation(&mut self) {
        let Screen::Onboarding(state) = &mut self.screen else {
            return;
        };
        let Some(rx) = &state.rx else { return };

        if let Ok(result) = rx.try_recv() {
            state.rx = None;
            match result {
                ValidationResult::Ok(key) => {
                    if let Err(e) = SecureStore::save_key(&key) {
                        state.status =
                            OnboardingStatus::Error(format!("Couldn't save key securely: {e}"));
                        return;
                    }
                    self.screen = Screen::Main(MainState { api_key: key });
                }
                ValidationResult::Err(msg) => {
                    state.status = OnboardingStatus::Error(msg);
                }
            }
        }
    }
}