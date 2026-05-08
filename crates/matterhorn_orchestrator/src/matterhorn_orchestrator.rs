use anyhow::{Result, bail};
use matterhorn_common::MatterhornConfig;
use serde::{Deserialize, Serialize};

/// Intent classification output from the intent parser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    /// Navigate to a URL in the dApp viewport.
    Navigate { url: String },
    /// Search for a natural-language query.
    Search { query: String },
    /// Execute a transaction (deferred to wallet confirmation).
    Transact { to: String, amount: String, token: String },
    /// Multi-step sequence of sub-intents.
    MultiStep { steps: Vec<SubIntent> },
    /// Could not classify the input.
    Unknown { raw: String },
}

/// A single step in a multi-step plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubIntent {
    pub description: String,
    pub intent: Box<Intent>,
}

/// LLM response envelope.
#[derive(Debug, Deserialize)]
struct ClassificationResponse {
    intent: String,
    entities: Option<IntentEntities>,
}

#[derive(Debug, Deserialize)]
struct IntentEntities {
    url: Option<String>,
    query: Option<String>,
    to: Option<String>,
    amount: Option<String>,
    token: Option<String>,
}

/// The orchestrator: takes raw input, classifies it, and returns an Intent.
pub struct MatterhornOrchestrator {
    config: MatterhornConfig,
    client: reqwest::Client,
}

impl MatterhornOrchestrator {
    pub fn new(config: MatterhornConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Parse raw composer input into a structured Intent using regex heuristics.
    pub async fn parse_input(&self, input: &str) -> Result<Intent> {
        Ok(self.parse_input_sync(input))
    }

    /// Synchronous version of parse_input for use in non-async contexts (render loops).
    pub fn parse_input_sync(&self, input: &str) -> Intent {
        if input.is_empty() {
            return Intent::Unknown { raw: input.into() };
        }

        if let Some(url) = extract_url(input) {
            return Intent::Navigate { url };
        }

        if let Some((to, amount, token)) = extract_transaction(input) {
            return Intent::Transact { to, amount, token };
        }

        if !contains_url_pattern(input) {
            return Intent::Search {
                query: input.trim().into(),
            };
        }

        Intent::Unknown { raw: input.into() }
    }

    /// Route a parsed intent to a human-readable action description.
    pub fn route(&self, intent: &Intent) -> String {
        match intent {
            Intent::Navigate { url } => format!("Opening {} in dApp viewport", url),
            Intent::Search { query } => format!("Searching for \"{}\"", query),
            Intent::Transact { to, amount, token } => {
                format!("Preparing to send {} {} to {}", amount, token, to)
            }
            Intent::MultiStep { steps } => {
                format!("Executing {} steps", steps.len())
            }
            Intent::Unknown { raw } => format!("Unrecognized input: \"{}\"", raw),
        }
    }

    /// Classify input using an LLM (OpenAI-compatible endpoint).
    pub async fn classify_with_llm(&self, input: &str) -> Result<Intent> {
        let prompt = serde_json::json!({
            "model": self.config.llm_model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Web3 intent parser. Classify user input into one of: navigate, search, transact, multi_step, unknown. Extract entities: url, query, to, amount, token. Return JSON with fields: intent, entities."
                },
                {
                    "role": "user",
                    "content": input
                }
            ],
            "temperature": 0.1,
            "max_tokens": 200
        });

        let response = self
            .client
            .post(&self.config.llm_endpoint)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.config.llm_api_key.as_deref().unwrap_or("")
                ),
            )
            .json(&prompt)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("LLM request failed ({status}): {body}");
        }

        let classification: ClassificationResponse = response.json().await?;
        Ok(classification_into_intent(classification, input))
    }
}

fn classification_into_intent(response: ClassificationResponse, raw_input: &str) -> Intent {
    match response.intent.as_str() {
        "navigate" => {
            let url = response
                .entities
                .as_ref()
                .and_then(|e| e.url.clone())
                .unwrap_or_else(|| raw_input.into());
            Intent::Navigate { url }
        }
        "search" => {
            let query = response
                .entities
                .as_ref()
                .and_then(|e| e.query.clone())
                .unwrap_or_else(|| raw_input.into());
            Intent::Search { query }
        }
        "transact" => Intent::Transact {
            to: response
                .entities
                .as_ref()
                .and_then(|e| e.to.clone())
                .unwrap_or_default(),
            amount: response
                .entities
                .as_ref()
                .and_then(|e| e.amount.clone())
                .unwrap_or_default(),
            token: response
                .entities
                .as_ref()
                .and_then(|e| e.token.clone())
                .unwrap_or_default(),
        },
        "multi_step" => Intent::MultiStep { steps: Vec::new() },
        _ => Intent::Unknown {
            raw: raw_input.into(),
        },
    }
}

fn extract_url(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let has_scheme = trimmed.starts_with("http://") || trimmed.starts_with("https://");
    let has_dot = trimmed.contains('.') && !trimmed.contains(' ');
    let is_address = trimmed.len() >= 40
        && trimmed.starts_with("0x")
        && trimmed[2..].chars().all(|c| c.is_ascii_hexdigit());

    if (has_scheme || (has_dot && !is_address && !trimmed.contains('\n'))) && trimmed.len() > 3 {
        let url = if has_scheme {
            trimmed.to_string()
        } else {
            format!("https://{}", trimmed)
        };
        return Some(url);
    }
    None
}

fn extract_transaction(input: &str) -> Option<(String, String, String)> {
    let trimmed = input.trim().to_lowercase();
    let has_action = trimmed.contains("send")
        || trimmed.contains("transfer")
        || trimmed.contains("swap")
        || trimmed.contains("bridge")
        || trimmed.contains("stake");

    if !has_action {
        return None;
    }

    let tokens = ["eth", "sol", "usdc", "usdt", "matic", "btc"];
    tokens.iter().find(|t| trimmed.contains(*t))?;

    let amount_pattern = r"(\d+\.?\d*)\s*(ETH|SOL|USDC|USDT|MATIC|BTC)";
    let amount_re = regex_lite::Regex::new(amount_pattern).ok()?;
    let input_upper = trimmed.to_uppercase();
    let captures = amount_re.captures(&input_upper)?;
    let amount: String = captures.get(1)?.as_str().into();
    let token: String = captures.get(2)?.as_str().to_lowercase();

    let to_pattern = r"(?:to|send|transfer)\s+(0x[a-fA-F0-9]{40}|[a-zA-Z0-9.-]+\.eth)";
    let to_re = regex_lite::Regex::new(to_pattern).ok()?;
    let to_match = to_re.captures(&trimmed);
    let to: String = match to_match {
        Some(caps) => caps.get(1).map(|m| m.as_str().into()).unwrap_or_else(|| "unknown".into()),
        None => "unknown".into(),
    };

    Some((to, amount, token))
}

fn contains_url_pattern(input: &str) -> bool {
    let pattern = r"https?://|\.(com|org|io|so|app|xyz|finance|eth|sol)";
    regex_lite::Regex::new(pattern)
        .map(|r| r.is_match(input))
        .unwrap_or(false)
}
