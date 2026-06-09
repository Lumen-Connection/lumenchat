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

pub struct ModelEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub descriptor: &'static str,
}

pub struct ModelGroup {
    pub provider: &'static str,
    pub models: &'static [ModelEntry],
}

pub const MODEL_GROUPS: &[ModelGroup] = &[
    ModelGroup {
        provider: "OpenAI",
        models: &[
            ModelEntry {
                id: "openai/gpt-5.5",
                name: "GPT-5.5",
                descriptor: "Heavy reasoning",
            },
            ModelEntry {
                id: "openai/gpt-5.4",
                name: "GPT-5.4 Thinking",
                descriptor: "Better thinking",
            },
            ModelEntry {
                id: "openai/gpt-5.3-chat",
                name: "GPT-5.3 Instant",
                descriptor: "Daily chat model",
            }
        ],
    },
    ModelGroup {
        provider: "Google",
        models: &[
            ModelEntry {
                id: "google/gemini-3.1-pro-preview",
                name: "Gemini 3.1 Pro",
                descriptor: "Deep thinking",
            },
            ModelEntry {
                id: "google/gemini-3.5-flash",
                name: "Gemini 3.5 Flash",
                descriptor: "Balanced model",
            },
            ModelEntry {
                id: "google/gemini-3.1-flash-lite",
                name: "Gemini 3.1 Flash-Lite",
                descriptor: "Cheap, quick thinker",
            }
        ],
    },
    ModelGroup {
        provider: "Anthropic",
        models: &[
            ModelEntry {
                id: "anthropic/claude-opus-4.8",
                name: "Claude Opus 4.8",
                descriptor: "State-of-the-art reasoning model"
            },
            ModelEntry {
                id: "anthropic/claude-sonnet-4.6",
                name: "Claude Sonnet 4.6",
                descriptor: "Balanced, adaptive model",
            },
            ModelEntry {
                id: "anthropic/claude-haiku-4.5",
                name: "Claude Haiku 4.5",
                descriptor: "Fast, efficient model",
            },
        ],
    },
    ModelGroup {
        provider: "xAI",
        models: &[
            ModelEntry {
                id: "x-ai/grok-4.20-multi-agent",
                name: "Grok 4.20 Multi-Agent",
                descriptor: "Uncensored multiagentic reasoning",
            },
            ModelEntry {
                id: "x-ai/grok-4.3",
                name: "Grok 4.3",
                descriptor: "Uncensored advanced reasoning",
            },
            ModelEntry {
                id: "x-ai/grok-build-0.1",
                name: "Grok Build 0.1",
                descriptor: "Fast agentic coding model",
            }
        ]
    },
    ModelGroup {
        provider: "Alibaba",
        models: &[
            ModelEntry {
                id: "qwen/qwen3.7-max",
                name: "Qwen3.7-Max",
                descriptor: "Extreme thinking model",
            },
            ModelEntry {
                id: "qwen/qwen3.7-plus",
                name: "Qwen3.7-Plus",
                descriptor: "Adaptive reasoning model",
            },
            ModelEntry {
                id: "qwen/qwen3.6-flash",
                name: "Qwen3.6-Flash",
                descriptor: "Cheap, quick model",
            },
        ]
    },
    ModelGroup {
        provider: "DeepSeek",
        models: &[
            ModelEntry {
                id: "deepseek/deepseek-v4-pro",
                name: "DeepSeek V4 Pro",
                descriptor: "DeepSeek's latest advanced model",
            },
            ModelEntry {
                id: "deepseek/deepseek-v4-flash",
                name: "DeepSeek V4 Flash",
                descriptor: "DeepSeek's latest fast model",
            },
            ModelEntry {
                id: "deepseek/deepseek-v3.2",
                name: "DeepSeek V3.2",
                descriptor: "DeepSeek's legacy model",
            },
        ]
    },
    ModelGroup {
        provider: "Z.ai",
        models: &[
            ModelEntry {
                id: "z-ai/glm-5.1",
                name: "GLM-5.1",
                descriptor: "Z.ai's latest reasoning model",
            },
            ModelEntry {
                id: "z-ai/glm-5v-turbo",
                name: "GLM-5V-Turbo",
                descriptor: "Z.ai's latest multimodal model",
            },
            ModelEntry {
                id: "z-ai/glm-5-turbo",
                name: "GLM-5-Turbo",
                descriptor: "Z.ai's previous reasoning model",
            },
        ]
    },
    ModelGroup {
        provider: "Moonshot AI",
        models: &[
            ModelEntry {
                id: "moonshotai/kimi-k2.6",
                name: "Kimi K2.6",
                descriptor: "Moonshot's latest reasoning model",
            },
            ModelEntry {
                id: "moonshotai/kimi-k2.5",
                name: "Kimi K2.5",
                descriptor: "Moonshot's previous reasoning model",
            },
        ]
    },
    ModelGroup {
        provider: "MiniMax",
        models: &[
            ModelEntry {
                id: "minimax/minimax-m3",
                name: "MiniMax-M3",
                descriptor: "MiniMax's latest agentic model"
            },
            ModelEntry {
                id: "minimax/minimax-m2.7",
                name: "MiniMax-M2.7",
                descriptor: "MiniMax's previous agentic model",
            },
            ModelEntry {
                id: "minimax/minimax-m2-her",
                name: "MiniMax-M2-her",
                descriptor: "MiniMax's conversational model",
            },
        ]
    },
    ModelGroup {
        provider: "Xiaomi",
        models: &[
            ModelEntry {
                id: "xiaomi/mimo-v2.5-pro",
                name: "MiMo-V2.5-Pro",
                descriptor: "Xiaomi's latest advanced model",
            },
            ModelEntry {
                id: "xiaomi/mimo-v2.5",
                name: "MiMo-V2.5",
                descriptor: "Xiaomi's latest balanced model",
            },
            ModelEntry {
                id: "xiaomi/mimo-v2-flash",
                name: "MiMo-V2-Flash",
                descriptor: "Xiaomi's previous ultra-low cost model",
            },
        ]
    },
    ModelGroup {
        provider: "Coding",
        models: &[
            ModelEntry {
                id: "openai/gpt-5.5-pro",
                name: "GPT-5.5 Pro",
                descriptor: "Extreme cost, ultra intelligent OpenAI model",
            },
            ModelEntry {
                id: "openai/gpt-5.3-codex",
                name: "GPT-5.3 Codex",
                descriptor: "OpenAI's coding-oriented model",
            },
            ModelEntry {
                id: "kwaipilot/kat-coder-pro-v2",
                name: "KAT-Coder-Pro V2",
                descriptor: "Kwai's latest advanced coding model",
            },
        ]
    },
    ModelGroup {
        provider: "FREE",
        models: &[
            ModelEntry {
                id: "moonshotai/kimi-k2.6:free",
                name: "Kimi K2.6",
                descriptor: "Free tier of Moonshot's latest LLM",
            },
            ModelEntry {
                id: "nvidia/nemotron-3-super-120b-a12b:free",
                name: "Nemotron 3 Super",
                descriptor: "Free tier of NVIDIA's latest LLM",
            },
            ModelEntry {
                id: "google/gemma-4-31b-it:free",
                name: "Gemma 4 31B",
                descriptor: "Free tier of Google's open source LLM",
            },
        ]
    },
];

pub const DEFAULT_MODEL: &str = "xiaomi/mimo-v2-flash";

pub fn find_model(id: &str) -> Option<&'static ModelEntry> {
    MODEL_GROUPS
        .iter()
        .flat_map(|g| g.models.iter())
        .find(|m| m.id == id)
}

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
    pub show_about: bool,
    pub focus_input_next_frame: bool,
    pub confirm_eject: bool,
}

impl MainState {
    pub fn active_chat_mut(&mut self) -> Option<&mut Chat> {
        if self.temporary_mode {
            self.temp_chat.as_mut()
        } 
        else {
            let id = self.active_chat_id?;
            self.chats.iter_mut().find(|c| c.id == id)
        }
    }

    pub fn active_chat(&self) -> Option<&Chat> {
        if self.temporary_mode {
            self.temp_chat.as_ref()
        } 
        else {
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
            show_about: false,
            focus_input_next_frame: false,
            confirm_eject: false,
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

    fn with_initial_chat(mut main: MainState) -> MainState {
        if main.chats.is_empty() {
            let chat = Chat::new(DEFAULT_MODEL.into());
            main.active_chat_id = Some(chat.id);
            main.chats.push(chat);
        }
        main
    }

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
        state.focus_input_next_frame = true;
    }

    pub fn select_chat(&mut self, id: Uuid) {
        let Screen::Main(state) = &mut self.screen else { return };
        if state.temporary_mode {
            return;
        }
        state.active_chat_id = Some(id);
        state.focus_input_next_frame = true;
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
        } 
        else {
            state.temp_chat = None;
        }
        state.focus_input_next_frame = true;
    }

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

    pub fn eject_key(&mut self) {
        if let Err(e) = SecureStore::delete_key() {
            eprintln!("[warn] couldn't delete cached key? {e:#}");
        }
        self.screen = Screen::Onboarding(OnboardingState {
            key_input: String::new(),
            show_key: false,
            status: OnboardingStatus::Idle,
            rx: None,
        });
    }

    pub fn poll_pending(&mut self) {
        let Screen::Main(state) = &mut self.screen else { return };
        let Some(pending) = &state.pending else { return };
        let Ok(result) = pending.rx.try_recv() else { return };
        let chat_id = pending.chat_id;
        state.pending = None;
        state.focus_input_next_frame = true;

        let target = if state.temporary_mode {
            state.temp_chat.as_mut().filter(|c| c.id == chat_id)
        } 
        else {
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
        } 
        else if !state.temporary_mode {
            let _ = storage::save_chats(&state.chats);
        }
    }
}