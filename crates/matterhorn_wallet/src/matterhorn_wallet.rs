use anyhow::{Result, bail};
use bip39::Mnemonic;
use bs58;
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use gpui::{Entity, Global};
use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;
use security_framework::os::macos::keychain::SecKeychain;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

const KEYCHAIN_SERVICE: &str = "com.matterhorn.browser.wallet";
const RPC_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    pub name: String,
    pub address: String,
    pub derivation_path: String,
}

/// A human-readable transaction plan for the confirmation sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub token: String,
}

pub struct MatterhornWallet {
    pub accounts: Vec<WalletAccount>,
    pub selected_account_index: usize,
    mnemonic: Option<Mnemonic>,
    signing_key: Option<SigningKey>,
    solana_key: Option<Ed25519SigningKey>,
    password_hash: Option<[u8; 32]>,
}

impl MatterhornWallet {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            selected_account_index: 0,
            mnemonic: None,
            signing_key: None,
            solana_key: None,
            password_hash: None,
        }
    }

    /// Check if a wallet is stored in the macOS Keychain.
    pub fn has_stored_wallet() -> bool {
        SecKeychain::default()
            .and_then(|keychain| keychain.find_generic_password(KEYCHAIN_SERVICE, "mnemonic"))
            .is_ok()
    }

    pub fn create(&mut self, password: &str) -> Result<String> {
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)?;
        let phrase = mnemonic.to_string();
        let seed = mnemonic.to_seed(password);
        let signing_key = SigningKey::from_slice(&seed[..32])?;
        let address = derive_ethereum_address(&signing_key);

        self.mnemonic = Some(mnemonic);
        self.signing_key = Some(signing_key);
        self.password_hash = Some(hash_password(password));

        self.accounts.push(WalletAccount {
            name: "Account 1".into(),
            address,
            derivation_path: "m/44'/60'/0'/0/0".into(),
        });
        self.selected_account_index = 0;
        Ok(phrase)
    }

    pub fn import(&mut self, phrase: &str, password: &str) -> Result<()> {
        let mnemonic = Mnemonic::parse_normalized(phrase)?;
        let seed = mnemonic.to_seed(password);
        let signing_key = SigningKey::from_slice(&seed[..32])?;
        let address = derive_ethereum_address(&signing_key);

        self.mnemonic = Some(mnemonic);
        self.signing_key = Some(signing_key);
        self.password_hash = Some(hash_password(password));

        self.accounts.push(WalletAccount {
            name: "Imported Account".into(),
            address,
            derivation_path: "m/44'/60'/0'/0/0".into(),
        });
        self.selected_account_index = 0;
        Ok(())
    }

    pub fn selected_address(&self) -> Option<&str> {
        self.accounts
            .get(self.selected_account_index)
            .map(|a| a.address.as_str())
    }

    pub fn selected_account(&self) -> Option<&WalletAccount> {
        self.accounts.get(self.selected_account_index)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        match self.password_hash {
            Some(stored) => hash_password(password) == stored,
            None => false,
        }
    }

    pub fn store_in_keychain(&self) -> Result<()> {
        let mnemonic = match &self.mnemonic {
            Some(m) => m,
            None => bail!("No mnemonic to store"),
        };
        let phrase = mnemonic.to_string();
        let keychain = SecKeychain::default()?;
        keychain.set_generic_password(KEYCHAIN_SERVICE, "mnemonic", phrase.as_bytes())?;
        Ok(())
    }

    pub fn load_from_keychain(&mut self, password: &str) -> Result<()> {
        let keychain = SecKeychain::default()?;
        let (phrase_bytes, _) =
            keychain.find_generic_password(KEYCHAIN_SERVICE, "mnemonic")?;
        let phrase = String::from_utf8(phrase_bytes.to_vec())?;
        self.import(&phrase, password)?;
        Ok(())
    }

    /// Resolve an ENS name to an Ethereum address using the ENS Ideas API.
    /// Returns None if resolution fails or the name is not registered.
    pub async fn resolve_ens(&self, _rpc_url: &str, name: &str) -> Option<String> {
        let url = format!("https://api.ensideas.com/ens/resolve/{}", name);
        match reqwest::get(&url).await {
            Ok(response) => {
                let body: serde_json::Value = response.json().await.ok()?;
                body["address"].as_str().map(|a| a.to_string())
            }
            Err(_) => None,
        }
    }

    /// Reverse resolve an Ethereum address to an ENS name using the ENS Ideas API.
    /// Returns None if resolution fails or the address has no ENS name.
    pub async fn resolve_ens_name(&self, address: &str) -> Option<String> {
        let url = format!("https://api.ensideas.com/ens/resolve/{}", address);
        match reqwest::get(&url).await {
            Ok(response) => {
                let body: serde_json::Value = response.json().await.ok()?;
                body["name"].as_str().map(|n| n.to_string())
            }
            Err(_) => None,
        }
    }

    pub async fn fetch_balance(&self, rpc_url: &str) -> Result<String> {
        let Some(address) = self.selected_address() else {
            bail!("No account selected");
        };
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [address, "latest"],
            "id": 1
        });
        let client = reqwest::Client::new();
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .timeout(RPC_REQUEST_TIMEOUT)
            .send()
            .await?;
        let body: serde_json::Value = response.json().await?;
        let hex_balance = body["result"]
            .as_str()
            .unwrap_or("0x0")
            .trim_start_matches("0x");
        let wei = u128::from_str_radix(hex_balance, 16)?;
        Ok(format_ether(wei))
    }

    pub fn sign_message(&self, message_hash: &[u8; 32]) -> Result<String> {
        let signing_key = match &self.signing_key {
            Some(k) => k,
            None => bail!("No signing key available"),
        };
        let (signature, _recovery_id) = signing_key.sign_prehash_recoverable(message_hash)?;
        Ok(hex::encode(signature.to_bytes()))
    }

    /// Build a human-readable transaction plan from parsed intent entities.
    pub fn build_transaction(
        &self,
        to: &str,
        amount: &str,
        token: &str,
    ) -> Result<TransactionRequest> {
        let from = match self.selected_address() {
            Some(a) => a.to_string(),
            None => bail!("No account selected"),
        };
        Ok(TransactionRequest {
            from,
            to: to.to_string(),
            amount: amount.to_string(),
            token: token.to_string(),
        })
    }

    /// Sign a transaction hash and return the signature hex.
    /// The caller is responsible for hashing the transaction RLP correctly.
    pub fn sign_transaction_hash(&self, tx_hash: &[u8; 32]) -> Result<String> {
        self.sign_message(tx_hash)
    }

    /// Create a Solana account derived from the same mnemonic seed using
    /// BIP44 path m/44'/501'/0'/0'. Derives an Ed25519 keypair from bytes
    /// 32..64 of the PBKDF2 seed (next 32 bytes after the Ethereum key).
    pub fn create_solana(&mut self, password: &str) -> Result<String> {
        let mnemonic = match &self.mnemonic {
            Some(m) => m,
            None => bail!("No mnemonic available — create or import a wallet first"),
        };
        let seed = mnemonic.to_seed(password);
        let solana_seed_bytes: &[u8] = seed.get(32..64).ok_or_else(|| {
            anyhow::anyhow!("Seed too short for Solana key derivation")
        })?;
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(solana_seed_bytes);
        let signing_key = Ed25519SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        let address = bs58::encode(verifying_key.as_bytes()).into_string();

        self.solana_key = Some(signing_key);

        self.accounts.push(WalletAccount {
            name: "Solana Account".into(),
            address: address.clone(),
            derivation_path: "m/44'/501'/0'/0'".into(),
        });

        Ok(address)
    }

    /// Fetch the SOL balance for the selected Solana account using a Solana RPC endpoint.
    /// Returns the balance in SOL as a formatted string.
    pub async fn fetch_solana_balance(&self, rpc_url: &str) -> Result<String> {
        let address = match self.selected_account() {
            Some(account) if account.derivation_path == "m/44'/501'/0'/0'" => {
                account.address.clone()
            }
            _ => {
                // Fallback: try to find any Solana account
                match self.accounts.iter().find(|a| a.derivation_path == "m/44'/501'/0'/0'") {
                    Some(account) => account.address.clone(),
                    None => bail!("No Solana account found"),
                }
            }
        };

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [address],
            "id": 1
        });
        let client = reqwest::Client::new();
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .timeout(RPC_REQUEST_TIMEOUT)
            .send()
            .await?;
        let body: serde_json::Value = response.json().await?;
        let lamports = body["result"]["value"]
            .as_u64()
            .unwrap_or(0);
        let sol = lamports as f64 / 1_000_000_000.0;
        Ok(format!("{:.9} SOL", sol))
    }
}

fn derive_ethereum_address(signing_key: &SigningKey) -> String {
    let verifying_key = signing_key.verifying_key();
    let public_key = verifying_key.to_encoded_point(false);
    // Strip the 0x04 SEC1 uncompressed prefix; Ethereum hashes the 64-byte
    // (X || Y) pubkey with Keccak-256 and takes the last 20 bytes (== bytes 12..32).
    let pubkey_bytes = &public_key.as_bytes()[1..];
    let hash = Keccak256::digest(pubkey_bytes);
    format!("0x{}", hex::encode(&hash[12..32]))
}

fn hash_password(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

fn format_ether(wei: u128) -> String {
    let whole = wei / 1_000_000_000_000_000_000;
    let frac = wei % 1_000_000_000_000_000_000;
    if frac == 0 {
        format!("{}.0 ETH", whole)
    } else {
        let frac_str = format!("{:018}", frac);
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{} ETH", whole, trimmed)
    }
}

pub struct WalletGlobal(pub Entity<MatterhornWallet>);

impl Global for WalletGlobal {}
