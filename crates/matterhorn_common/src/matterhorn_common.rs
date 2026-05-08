use serde::{Deserialize, Serialize};

/// Central error type for Matterhorn Browser.
/// Wraps all subsystem errors into a single type for uniform handling.
#[derive(Debug)]
pub enum MatterhornError {
    Config(String),
    Wallet(String),
    Orchestrator(String),
    Viewport(String),
    Network(String),
    Io(std::io::Error),
}

impl std::fmt::Display for MatterhornError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(message) => write!(f, "config error: {message}"),
            Self::Wallet(message) => write!(f, "wallet error: {message}"),
            Self::Orchestrator(message) => write!(f, "orchestrator error: {message}"),
            Self::Viewport(message) => write!(f, "viewport error: {message}"),
            Self::Network(message) => write!(f, "network error: {message}"),
            Self::Io(error) => write!(f, "io error: {error}"),
        }
    }
}

impl std::error::Error for MatterhornError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MatterhornError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Application configuration loaded from settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatterhornConfig {
    /// LLM provider endpoint (OpenAI-compatible)
    pub llm_endpoint: String,
    /// LLM model name
    pub llm_model: String,
    /// API key for the LLM provider
    pub llm_api_key: Option<String>,
    /// Default Ethereum RPC endpoint
    pub ethereum_rpc: String,
    /// Default Solana RPC endpoint
    pub solana_rpc: String,
}

impl Default for MatterhornConfig {
    fn default() -> Self {
        Self {
            llm_endpoint: "https://api.openai.com/v1".into(),
            llm_model: "gpt-4o".into(),
            llm_api_key: None,
            ethereum_rpc: "https://eth.llamarpc.com".into(),
            solana_rpc: "https://api.mainnet-beta.solana.com".into(),
        }
    }
}
