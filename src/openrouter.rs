use reqwest::{Client, RequestBuilder, StatusCode};
use serde::Deserialize;

const BASE_URL: &str = "https://openrouter.ai/api/v1";

#[derive(Debug, thiserror::Error)]
pub enum OpenRouterError {
    #[error("API key was rejected by OpenRouter")]
    Unauthorized,

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("unexpected response from OpenRouter (HTTP {status}): {body}")]
    Unexpected { status: u16, body: String },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct KeyInfo {
    pub label: Option<String>,
    pub usage: Option<f64>,
    pub limit: Option<f64>,
    pub is_free_tier: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct KeyInfoEnvelope {
    data: KeyInfo,
}

pub struct OpenRouterClient {
    http: Client,
    api_key: String,
}

impl OpenRouterClient {
    pub fn new(api_key: String) -> Result<Self, reqwest::Error> {
        let http = Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { http, api_key })
    }

    fn auth_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.bearer_auth(&self.api_key)
    }

    pub async fn validate_key(&self) -> Result<KeyInfo, OpenRouterError> {
        let req = self.http.get(format!("{}/key", BASE_URL));
        let resp = self.auth_headers(req).send().await?;

        let status = resp.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(OpenRouterError::Unauthorized);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OpenRouterError::Unexpected {
                status: status.as_u16(),
                body,
            });
        }

        let envelope: KeyInfoEnvelope = resp.json().await?;
        Ok(envelope.data)
    }
}