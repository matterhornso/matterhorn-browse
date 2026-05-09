use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

impl MatterhornConfig {
    /// Path to the config file: `~/.matterhorn/config.json`. Returns `None`
    /// when `$HOME` cannot be resolved.
    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".matterhorn").join("config.json"))
    }

    /// Load the config from disk, falling back to `Default::default()` when
    /// the file is missing, unreadable, or fails to parse. Errors are logged
    /// to stderr but not propagated — first-launch always uses defaults.
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(Some(config)) => config,
            Ok(None) => Self::default(),
            Err(e) => {
                eprintln!("matterhorn: failed to load config, using defaults: {e}");
                Self::default()
            }
        }
    }

    /// Read the config from disk. `Ok(None)` means the file doesn't exist yet.
    pub fn load() -> Result<Option<Self>, MatterhornError> {
        let Some(path) = Self::config_path() else {
            return Ok(None);
        };
        match std::fs::read(&path) {
            Ok(bytes) => {
                let config = serde_json::from_slice(&bytes)
                    .map_err(|e| MatterhornError::Config(format!("parse config: {e}")))?;
                Ok(Some(config))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MatterhornError::Io(e)),
        }
    }

    /// Atomically write the config to disk, creating `~/.matterhorn/` if it
    /// does not already exist.
    pub fn save(&self) -> Result<(), MatterhornError> {
        let path = Self::config_path()
            .ok_or_else(|| MatterhornError::Config("$HOME not set".into()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| MatterhornError::Config(format!("serialize config: {e}")))?;
        std::fs::write(&path, bytes)?;
        Ok(())
    }
}
