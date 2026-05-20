use crate::openrouter::{ChatMessage, OpenRouterClient, OpenRouterError};
use crate::secure_store::SecureStore;
use crate::storage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;
use uuid::Uuid;

pub const AVAILABLE_MODELS: &[(&str, &str)] = &[
    ("openai/gpt-5.4-mini", "GPT-5.4 mini"),
    ("anthropic/claude-sonnet-4.6", "Claude Sonnet 4.6"),
    ("anthropic/claude-haiku-4.5", "Claude Haiku 4.5"),
    ("google/gemini-3.1-flash-lite", "Gemini 3.1 Flash-Lite"),
    ("deepseek/deepseek-v4-flash:free", "DeepSeek V4 Flash"),
];

pub const DEFAULT_MODEL: &str = "deepseek/deepseek-v4-flash:free";

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
    Ok(String),
    Err(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// When this message first appeared in the UI. Used for the fade-in animation.
    /// Not serialized — restored messages should appear instantly.
    #[serde(skip, default = "Instant::now")]
    pub appeared_at: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: Uuid,
    pub title: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
}

impl Chat {
    pub fn new(model: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "New chat".into(),
            model,
            messages: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

pub struct PendingResponse {
    pub chat_id: Uuid,
    pub rx: Receiver<Result<String, String>>,
}

pub struct MainState {
    pub client: OpenRouterClient,
    pub chats: Vec<Chat>,
    pub active_chat_id: Option<Uuid>,
    pub temp_chat: Option<Chat>,
    pub temporary_mode: bool,
    pub input: String,
    pub pending: Option<PendingResponse>,
}

impl MainState {
    /// Return a mutable reference to whichever chat is currently active —
    /// either the temporary one (if temp mode is on) or one from the saved list.
    pub fn active_chat_mut(&mut self) -> Option<&mut Chat> {
        if self.temporary_mode {
            self.temp_chat.as_mut()
        } else {
            let id = self.active_chat_id?;
            self.chats.iter_mut().find(|c| c.id == id)
        }
    }

    pub fn active_chat(&self) -> Option<&Chat> {
        if self.temporary_mode {
            self.temp_chat.as_ref()
        } else {
            let id = self.active_chat_id?;
            self.chats.iter().find(|c| c.id == id)
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub rt: Arc<Runtime>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let rt = Arc::new(Runtime::new()?);

        let screen = match SecureStore::load_key()? {
            Some(key) => Screen::Main(Self::build_main_state(key)?),
            None => Screen::Onboarding(OnboardingState {
                key_input: String::new(),
                show_key: false,
                status: OnboardingStatus::Idle,
                rx: None,
            }),
        };

        Ok(Self { screen, rt })
    }

    fn build_main_state(api_key: String) -> anyhow::Result<MainState> {
        let client = OpenRouterClient::new(api_key.clone())?;
        let chats = storage::load_chats().unwrap_or_else(|e| {
            eprintln!("[warn] couldn't load chats.json: {e:#}");
            Vec::new()
        });
        let active_chat_id = chats.first().map(|c| c.id);

        Ok(MainState {
            client,
            chats,
            active_chat_id,
            temp_chat: None,
            temporary_mode: false,
            input: String::new(),
            pending: None,
        })
    }

    pub fn start_validation(&mut self) {
        let Screen::Onboarding(state) = &mut self.screen else {
            return;
        };
        let key = state.key_input.trim().to_string();
        if key.is_empty() {
            state.status = OnboardingStatus::Error("Please enter an API key.".into());
            return;
        }

        let (tx, rx): (Sender<ValidationResult>, Receiver<ValidationResult>) =
            mpsc::channel();
        state.rx = Some(rx);
        state.status = OnboardingStatus::Validating;

        let rt = self.rt.clone();
        let key_for_task = key.clone();
        rt.spawn(async move {
            let client = match OpenRouterClient::new(key_for_task.clone()) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(ValidationResult::Err(format!(
                        "Couldn't build HTTP client: {e}"
                    )));
                    return;
                }
            };
            match client.validate_key().await {
                Ok(_) => {
                    let _ = tx.send(ValidationResult::Ok(key_for_task));
                }
                Err(OpenRouterError::Unauthorized) => {
                    let _ = tx.send(ValidationResult::Err(
                        "That key was rejected by OpenRouter.".into(),
                    ));
                }
                Err(e) => {
                    let _ = tx.send(ValidationResult::Err(format!("{e}")));
                }
            }
        });
    }

    pub fn poll_validation(&mut self) {
        let Screen::Onboarding(state) = &mut self.screen else {
            return;
        };
        let Some(rx) = &state.rx else { return };
        let Ok(result) = rx.try_recv() else { return };
        state.rx = None;

        match result {
            ValidationResult::Ok(key) => {
                if let Err(e) = SecureStore::save_key(&key) {
                    state.status =
                        OnboardingStatus::Error(format!("Couldn't save key securely: {e}"));
                    return;
                }
                match Self::build_main_state(key) {
                    Ok(main) => self.screen = Screen::Main(Self::with_initial_chat(main)),
                    Err(e) => {
                        state.status =
                            OnboardingStatus::Error(format!("Couldn't initialize app: {e}"));
                    }
                }
            }
            ValidationResult::Err(msg) => {
                state.status = OnboardingStatus::Error(msg);
            }
        }
    }

    /// If there are no saved chats yet, make sure we start with a fresh one selected.
    fn with_initial_chat(mut main: MainState) -> MainState {
        if main.chats.is_empty() {
            let chat = Chat::new(DEFAULT_MODEL.into());
            main.active_chat_id = Some(chat.id);
            main.chats.push(chat);
        }
        main
    }

    /// Create a new chat and select it. Persists immediately.
    pub fn new_chat(&mut self) {
        let Screen::Main(state) = &mut self.screen else { return };
        if state.temporary_mode {
            state.temp_chat = Some(Chat::new(DEFAULT_MODEL.into()));
            return;
        }
        let chat = Chat::new(DEFAULT_MODEL.into());
        state.active_chat_id = Some(chat.id);
        state.chats.insert(0, chat);
        let _ = storage::save_chats(&state.chats);
    }

    pub fn select_chat(&mut self, id: Uuid) {
        let Screen::Main(state) = &mut self.screen else { return };
        if state.temporary_mode {
            return;
        }
        state.active_chat_id = Some(id);
    }

    pub fn delete_chat(&mut self, id: Uuid) {
        let Screen::Main(state) = &mut self.screen else { return };
        state.chats.retain(|c| c.id != id);
        if state.active_chat_id == Some(id) {
            state.active_chat_id = state.chats.first().map(|c| c.id);
        }
        if state.chats.is_empty() && !state.temporary_mode {
            let chat = Chat::new(DEFAULT_MODEL.into());
            state.active_chat_id = Some(chat.id);
            state.chats.push(chat);
        }
        let _ = storage::save_chats(&state.chats);
    }

    pub fn set_temporary_mode(&mut self, on: bool) {
        let Screen::Main(state) = &mut self.screen else { return };
        if state.temporary_mode == on {
            return;
        }
        state.temporary_mode = on;
        if on {
            state.temp_chat = Some(Chat::new(DEFAULT_MODEL.into()));
        } else {
            state.temp_chat = None;
        }
    }

    /// Send the current input on the active chat.
    pub fn send_message(&mut self) {
        let Screen::Main(state) = &mut self.screen else { return };
        if state.pending.is_some() {
            return;
        }
        let text = state.input.trim().to_string();
        if text.is_empty() {
            return;
        }

        let Some(chat) = state.active_chat_mut() else { return };
        let chat_id = chat.id;
        let model = chat.model.clone();

        chat.messages.push(Message {
            role: Role::User,
            content: text.clone(),
            appeared_at: Instant::now(),
        });

        // Auto-title from first user message.
        if chat.title == "New chat" {
            chat.title = text.chars().take(40).collect::<String>();
            if text.chars().count() > 40 {
                chat.title.push('…');
            }
        }

        let history: Vec<ChatMessage> = chat
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: match m.role {
                    Role::User => "user".into(),
                    Role::Assistant => "assistant".into(),
                },
                content: m.content.clone(),
            })
            .collect();

        state.input.clear();

        if !state.temporary_mode {
            let _ = storage::save_chats(&state.chats);
        }

        let (tx, rx) = mpsc::channel::<Result<String, String>>();
        state.pending = Some(PendingResponse { chat_id, rx });

        let client = state.client.clone();
        self.rt.spawn(async move {
            let result = client
                .chat_completion(&model, &history)
                .await
                .map_err(|e| format!("{e}"));
            let _ = tx.send(result);
        });
    }

    pub fn poll_pending(&mut self) {
        let Screen::Main(state) = &mut self.screen else { return };
        let Some(pending) = &state.pending else { return };
        let Ok(result) = pending.rx.try_recv() else { return };
        let chat_id = pending.chat_id;
        state.pending = None;

        // Find the chat the response belongs to. It might be the temp chat,
        // a saved chat, or it may have been deleted while we waited.
        let target = if state.temporary_mode {
            state.temp_chat.as_mut().filter(|c| c.id == chat_id)
        } else {
            state.chats.iter_mut().find(|c| c.id == chat_id)
        };
        let Some(chat) = target else { return };

        let (content, ok) = match result {
            Ok(text) => (text, true),
            Err(e) => (format!("⚠ Error: {e}"), false),
        };

        chat.messages.push(Message {
            role: Role::Assistant,
            content,
            appeared_at: Instant::now(),
        });

        if !state.temporary_mode && ok {
            let _ = storage::save_chats(&state.chats);
        } else if !state.temporary_mode {
            // Persist the error too so the user has a record.
            let _ = storage::save_chats(&state.chats);
        }
    }
}