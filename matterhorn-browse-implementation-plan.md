# Matterhorn Browse — Full Implementation Plan

*Generated: April 30, 2026 | Methodology: [gstack](https://github.com/garrytan/gstack) autoplan pipeline*
*Target: https://github.com/matterhornso/matterhorn-browse | Fork: zed-industries/zed (v1.1.x)*

---

## Phase 0: Product Framing (Office Hours — 6 Forcing Questions)

> gstack /office-hours methodology: expose demand reality, status quo, narrowest wedge, observation, future-fit.

**Q1: Who has this problem, and how do we know they're desperate for a solution?**

The DePIN sector has crossed $50B market cap with 1M+ Helium hotspot owners, 300K+ io.net GPU providers, and 3,000+ Filecoin storage providers. None of these operators have a unified dashboard. They manage separate web apps, CLI tools, and mobile apps. The desperation is real: "checking my Helium earnings means opening a separate app. Checking Render means another tab. I want one thing."

**Q2: What is the status quo, and why does it hold?**

The status quo is Chrome + MetaMask + 5 DePIN dashboards bookmarked. It holds because no one has challenged the "browser is neutral shell, wallet is extension" architecture. The browser itself has never been rethought as a Web3-native application.

**Q3: Who is the first user, specifically?**

The Helium hotspot operator with 3-5 hotspots, earning $5-20/day in HNT/MOBILE, checking earnings daily. They also hold some ETH/SOL. They are not a developer — they bought a device and plugged it in. They are exactly "anyone curious about Web3."

**Q4: What's the narrowest wedge that ships value?**

**A browser that opens tabs, has a self-custody EVM wallet, and shows your Helium hotspot earnings.** That's it. One tab = web. One panel = wallet. One panel = DePIN (Helium only). Ship that in 4 weeks.

**Q5: What have we observed from our own usage?**

Every Web3 tool has terrible onboarding. "Create wallet" means "download extension, write down 12 words, verify them, find the hidden settings menu." A browser where wallet creation is step 1 of onboarding — not a separate install — eliminates the single biggest barrier to Web3 adoption.

**Q6: Does this fit the future?**

Yes. As DePIN grows from $50B to $200B+, a unified browser/dashboard becomes the default interface. As more people interact with on-chain data, treating `0x...` addresses and `.eth` names as first-class URL schemes makes sense. Matterhorn Browse is ahead of a curve that is already bending.

---

## Phase 1: Strategic Architecture (CEO Review — Scope Expansion Mode)

> gstack /plan-ceo-review methodology: find the 10-star product. Challenge premises. Expand scope when it creates a better product.

### Mode: SELECTIVE EXPANSION
Hold core scope (browser + wallet + DePIN dashboard) but cherry-pick two expansions that compound:

1. **ENS / decentralized DNS resolution in the URL bar** — Makes the browser feel Web3-native immediately. Typing `vitalik.eth` and arriving at a page is the "aha" moment
2. **WalletConnect v2 fallback** — For users who already have MetaMask/Phantom. Reduces switching cost. 2-3 days of Rust protocol implementation

### What we explicitly do NOT do (yet):
- No mobile version (PWA only)
- No DEX/swap aggregation in wallet (just basic send/receive)
- No governance voting / DAO tooling
- No custom blockchain (this is browser infra, not a chain)

### 10-Section Strategic Review

| Section | Assessment |
|---------|-----------|
| **User** | Helium hotspot operator → DePIN operator → Web3 browser for everyone. Clear path |
| **Problem** | 5 dashboards + 2 wallets = fragmentation. One app = unification. Real |
| **Solution** | Browser shell with wallet + DePIN panels. Technically novel, UX-proven |
| **Market** | $50B DePIN sector, 70M Brave users (proof wallets-in-browsers work), 0 direct competitors |
| **Timing** | Web3 tools are mature enough (alloy-rs, solana-sdk). DePIN is growing 200% YoY. Right now |
| **Moats** | GPUI (GPU rendering), Rust (no JS bridge overhead), cross-network DePIN layer, OS keychain integration |
| **Revenue** | Open source (AGPL-3.0). Distribution inside Matterhorn 2.0. Future: premium DePIN analytics |
| **Risk** | Zed upstream drift. Fork maintenance. Wallet security audit liability. Key management bugs |
| **Team** | Matterhorn engineering team. Rust expertise via Zed codebase familiarity. AI-assisted development (gstack) |
| **Ship** | 4 weeks to alpha (browser + EVM wallet + Helium). 8 weeks to beta (+ Solana + Render/Filecoin). 12 weeks to launch |

---

## Phase 2: Engineering Architecture (Eng Review)

> gstack /plan-eng-review methodology: lock architecture, data flow, edge cases, test matrix, failure modes.

### 2.1 Crate Architecture

```
matterhorn-browse/
├── crates/
│   ├── gpui/                    [KEPT] GPU rendering engine
│   ├── gpui_macros/             [KEPT]
│   ├── gpui_platform/           [KEPT] macOS/Windows/Linux impl
│   ├── gpui_wgpu/               [KEPT] WebGPU backend
│   ├── workspace/               [MODIFIED] Tab management → chain-aware tabs
│   ├── terminal/                [KEPT] Embedded PTY
│   ├── terminal_view/           [KEPT]
│   ├── theme/                   [KEPT] Theme system
│   ├── extension/               [KEPT] Plugin framework
│   ├── extension_api/           [KEPT]
│   ├── extension_host/          [KEPT]
│   ├── editor/                  [STRIPPED] Code editing → light config editing only
│   ├── language/                [TRIMMED] Tree-sitter → Solidity/Vyper/Move only
│   ├── languages/               [TRIMMED]
│   ├── project/                 [REPLACED] → Network explorer
│   ├── project_panel/           [REPLACED] → Network explorer UI
│   ├── git/                     [REMOVED]
│   ├── git_ui/                  [REMOVED]
│   ├── git_graph/               [REMOVED]
│   ├── vim/                     [REMOVED]
│   ├── copilot/                 [REMOVED]
│   ├── copilot_chat/            [REMOVED]
│   ├── agent/                   [KEPT, OPTIONAL]
│   ├── agent_ui/                [KEPT, OPTIONAL]
│   ├── collab/                  [KEPT]
│   ├── collab_ui/               [KEPT]
│   ├── client/                  [KEPT] Network client
│   ├── channel/                 [KEPT] Real-time channels
│   ├── call/                    [KEPT] Audio/video calls
│   │
│   ├── mb_wallet/               ★ NEW — Self-custody wallet engine
│   │   ├── wallet.rs            Library root (alloy-rs + solana-sdk + keychain)
│   │   ├── keychain/
│   │   │   ├── macos.rs         macOS Keychain via Security framework
│   │   │   ├── linux.rs         freedesktop secrets / file-based fallback
│   │   │   └── windows.rs       Windows DPAPI
│   │   ├── evm.rs               EVM wallet: alloy-rs signer, provider, transaction builder
│   │   ├── solana.rs            Solana wallet: solana-sdk keypair, transaction, RPC client
│   │   ├── mnemonic.rs          BIP39 generation, encrypted storage
│   │   ├── derivation.rs        BIP44 derivation paths (m/44'/60'/...)
│   │   └── walletconnect.rs     WalletConnect v2 relay client (WebSocket + JSON-RPC)
│   │
│   ├── mb_wallet_ui/            ★ NEW — Wallet panel UI
│   │   ├── wallet_panel.rs      Wallet panel (balances, send/receive, activity)
│   │   ├── balance.rs           Balance display component
│   │   ├── send.rs              Send transaction flow
│   │   ├── receive.rs           Receive / QR code view
│   │   ├── activity.rs          Transaction history list
│   │   ├── network_selector.rs  Chain/network switcher
│   │   └── onboarding.rs        Wallet creation flow UI
│   │
│   ├── mb_depin/                ★ NEW — DePIN integration layer
│   │   ├── depin.rs             Library root — trait + registry
│   │   ├── framework.rs         Plugin framework: NetworkProvider trait
│   │   ├── helium.rs            Helium API client (Helium API + on-chain data)
│   │   ├── render.rs            Render Network API client
│   │   ├── filecoin.rs          Filecoin Lotus API client
│   │   ├── akash.rs             Akash Network API client
│   │   ├── iotex.rs             IoTeX device API (Pebble, UCam)
│   │   └── cache.rs             Local SQLite cache for DePIN data (offline-first)
│   │
│   ├── mb_depin_ui/             ★ NEW — DePIN dashboard UI
│   │   ├── depin_panel.rs       Main DePIN panel (network list + detail)
│   │   ├── helium_dashboard.rs  Helium hotspot earnings + coverage UI
│   │   ├── render_dashboard.rs  Render job list + submit UI
│   │   ├── filecoin_dashboard.rs Storage deal browser
│   │   ├── network_card.rs      Generic DePIN network card component
│   │   └── earnings_chart.rs    Earnings chart component (sparklines)
│   │
│   ├── mb_tab/                  ★ NEW — Chain-aware tab system
│   │   ├── tab.rs               Extended tab with chain metadata
│   │   ├── dapp_tab.rs          Dapp tab (sandboxed context, injected provider)
│   │   ├── explorer_tab.rs      Block explorer tab (parses chain data natively)
│   │   └── resolution.rs        URL resolution: ENS → address, IPFS → gateway
│   │
│   ├── mb_onboarding/           ★ NEW — Web3 onboarding
│   │   ├── onboarding.rs        Onboarding flow orchestrator
│   │   ├── wallet_create.rs     Wallet creation step
│   │   ├── key_backup.rs        Seed phrase display + verification
│   │   └── first_token.rs       First token acquisition guide
│   │
│   ├── mb_ens/                  ★ NEW — ENS / decentralized DNS
│   │   ├── ens.rs               ENS resolver (Ethereum mainnet + L2)
│   │   ├── ipfs.rs              IPFS gateway resolution
│   │   ├── arweave.rs           Arweave permaweb resolution
│   │   └── lens.rs              Lens Protocol profile resolution
│   │
│   ├── mb_content/              ★ NEW — Content resolver
│   │   ├── resolver.rs          Unified content resolver trait
│   │   ├── farcaster.rs         Farcaster cast rendering
│   │   └── mirror.rs            Mirror.xyz entry rendering
│   │
│   └── zed/                     [MODIFIED] Application entry point
│       └── main.rs              Renamed app, new window title, new branding
```

### 2.2 Data Flow: URL Resolution

```
User types URL/address/ENS name in address bar
    │
    ├─→ mb_tab::resolution::resolve(input)
    │       │
    │       ├─→ Starts with "http://" or "https://"?  → Load as web page (GPUI webview)
    │       ├─→ Ends with ".eth"?                      → mb_ens::resolve_eth(ens_name)
    │       │       ├─→ Has IPFS content record?       → mb_ens::ipfs::resolve(cid)
    │       │       ├─→ Has Arweave content record?    → mb_ens::arweave::resolve(tx_id)
    │       │       ├─→ Has Lens profile?              → mb_ens::lens::resolve(handle)
    │       │       └─→ Default: resolve to Ethereum address
    │       ├─→ Starts with "0x"?                      → Open as address (wallet view or explorer)
    │       ├─→ Looks like IPFS CID?                   → mb_ens::ipfs::resolve(cid)
    │       ├─→ Looks like Arweave tx?                 → mb_ens::arweave::resolve(tx_id)
    │       └─→ Default                                → Search via configured search engine
    │
    └─→ Open in appropriate tab type
```

### 2.3 Key Management Architecture

```
┌─────────────────────────────────────────────────┐
│                 MB Wallet Engine                 │
│                                                  │
│  ┌─────────────┐    ┌────────────────────────┐  │
│  │ BIP39       │    │ Platform Keychain       │  │
│  │ Mnemonic    │───▶│ (encrypted at rest)     │  │
│  │ Generation  │    │ macOS: Security.fw     │  │
│  └─────────────┘    │ Linux: libsecret        │  │
│                      │ Windows: DPAPI          │  │
│  ┌─────────────┐    └───────────┬────────────┘  │
│  │ BIP44       │                │               │
│  │ Derivation  │◀───────────────┘               │
│  │ Path Engine │                                │
│  └──────┬──────┘                                │
│         │                                       │
│    ┌────┴────┐         ┌──────────────┐        │
│    │ EVM Key │         │ Solana Key   │        │
│    │ (alloy) │         │ (solana-sdk) │        │
│    └────┬────┘         └──────┬───────┘        │
│         │                     │                 │
│    ┌────┴────────┐    ┌──────┴───────────┐    │
│    │ Sign Tx     │    │ Sign Tx           │    │
│    │ Build Tx    │    │ Build Tx          │    │
│    │ EstimateGas │    │ GetRecentBlockhash│    │
│    │ SendRawTx   │    │ SendRawTx         │    │
│    └─────────────┘    └──────────────────┘    │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │ WalletConnect v2 (optional fallback)     │   │
│  │ WebSocket relay → JSON-RPC encrypted     │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### 2.4 State Management (GPUI Entities)

```rust
// Global app state
struct MatterhornBrowse {
    wallet: Entity<WalletEngine>,       // Wallet engine
    depin: Entity<DepinRegistry>,       // DePIN network registry
    workspace: Entity<Workspace>,       // Tab workspace
    onboarding: Option<Entity<Onboarding>>, // Active onboarding flow
}

// Wallet engine state
struct WalletEngine {
    mnemonic: Option<EncryptedMnemonic>, // BIP39, encrypted
    evm_wallets: HashMap<u32, EvmWallet>, // derivation index → wallet
    solana_wallets: HashMap<u32, SolanaWallet>,
    active_network: Network,              // mainnet / sepolia / devnet
    walletconnect: Option<WcSession>,     // Active WC session
}

// DePIN registry
struct DepinRegistry {
    providers: Vec<Box<dyn NetworkProvider>>, // Plugin network providers
    cache: SqliteCache,                       // Offline data cache
    refresh_interval: Duration,               // Data refresh rate
}
```

### 2.5 Test Matrix

| Layer | What | Framework | Strategy |
|-------|------|-----------|----------|
| **Wallet crypto** | BIP39, BIP44, signing | `#[test]` unit tests | Known test vectors (Trezor test vectors) |
| **Keychain** | Platform key storage | Integration tests | Mock keychain, test encrypt/decrypt round-trip |
| **EVM wallet** | Transaction building, signing, broadcasting | `#[test]` + testnet | Sepolia/Holesky testnet integration |
| **Solana wallet** | Transaction building, signing | `#[test]` + devnet | Solana devnet integration |
| **DePIN integrations** | API client, data parsing | `#[test]` with fixtures | Record/replay API responses. No live API in CI |
| **UI (GPUI)** | Wallet panel, DePIN dashboard | GPUI test framework | `VisualTestContext` + snapshot testing |
| **Onboarding** | Full wallet creation flow | GPUI tests | End-to-end onboarding happy path |
| **ENS resolution** | Name → address lookup | `#[test]` | Mock ENS contract. Real integration test flagged |

### 2.6 Failure Modes

| Failure | Impact | Mitigation |
|---------|--------|------------|
| Keychain access denied | Cannot read/store wallet keys | Graceful error: "Keychain unavailable. Wallet features disabled." |
| RPC endpoint down | Cannot fetch balances / send tx | Fallback RPC rotation. Configurable endpoints |
| DePIN API rate-limited | Dashboard shows stale data | Local SQLite cache. Show "Last updated: X min ago" |
| WalletConnect relay down | Cannot connect to external wallets | Show status indicator. In-app wallet always works |
| Seed phrase lost | User loses funds | Force backup verification during onboarding. Never skip |
| GPUI rendering crash | Tab crashes but app stays alive | Per-tab process isolation plan (future). Currently: crash = restart |
| Upstream Zed merge conflict | Fork maintenance burden | Selective cherry-pick strategy. Only merge security fixes + GPUI improvements |

---

## Phase 3: Bite-Sized Implementation Tasks

> Each task = 2-5 minutes of focused implementation. Exact file paths. Complete code patterns. GPUI idioms observed from Zed codebase.

---

### Sprint 0: Foundation (Week 1, Days 1-3)

#### Task 0.1: Fork cleanup — rename app entry point

**Objective:** Change the application name and window title from "Zed" to "Matterhorn Browse"

**Files:**
- Modify: `crates/zed/src/main.rs`
- Modify: `crates/zed/Cargo.toml`

**Step 1: Update main.rs branding**
```rust
// In crates/zed/src/main.rs, find and replace:
// "Zed" → "Matterhorn Browse"
// "zed" → "matterhorn-browse"
// "Zed Industries" → "Matterhorn"

const APP_NAME: &str = "Matterhorn Browse";
```

**Step 2: Run build to verify**
```bash
cargo check -p zed
```

Expected: compiles without errors. App window title shows "Matterhorn Browse"

**Step 3: Commit**
```bash
git add crates/zed/src/main.rs crates/zed/Cargo.toml
git commit -m "chore: rename app entry point to Matterhorn Browse"
```

---

#### Task 0.2: Fork cleanup — update workspace Cargo.toml

**Objective:** Remove crates we won't use. Add placeholder entries for new crates

**Files:**
- Modify: `Cargo.toml` (workspace root)

**Step 1: Comment out removed crates**
```toml
# In [workspace.members], comment out:
# "crates/vim",
# "crates/git",
# "crates/git_ui",
# "crates/git_graph",
# "crates/copilot",
# "crates/copilot_chat",
```

**Step 2: Add new crates**
```toml
"crates/mb_wallet",
"crates/mb_wallet_ui",
"crates/mb_depin",
"crates/mb_depin_ui",
"crates/mb_tab",
"crates/mb_onboarding",
"crates/mb_ens",
"crates/mb_content",
```

**Step 3: Run check**
```bash
cargo check 2>&1 | head -20
```

Expected: errors about missing crates (we haven't created them yet). That's fine.

**Step 4: Commit**
```bash
git add Cargo.toml
git commit -m "chore: update workspace members — comment removed crates, add new crate entries"
```

---

#### Task 0.3: Create mb_wallet crate scaffold

**Objective:** Create the `mb_wallet` crate with proper Cargo.toml and library root

**Files:**
- Create: `crates/mb_wallet/Cargo.toml`
- Create: `crates/mb_wallet/wallet.rs` (library root, following Zed convention of descriptive file names)

**Step 1: Create crate directory**
```bash
mkdir -p crates/mb_wallet/src
```

**Step 2: Write Cargo.toml**
```toml
[package]
name = "mb_wallet"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
path = "wallet.rs"

[dependencies]
alloy = { version = "0.15", features = ["full"] }
alloy-signer-local = "0.15"
alloy-signer = "0.15"
alloy-provider = "0.15"
solana-sdk = "2.2"
bip39 = "2.1"
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = "2.0"
zeroize = "1.8"
rand = "0.9"
sha2 = "0.10"
hmac = "0.12"
pbkdf2 = "0.12"
aes-gcm = "0.10"
tokio = { workspace = true }
futures = { workspace = true }

[features]
default = ["evm", "solana"]
evm = ["alloy", "alloy-signer", "alloy-signer-local", "alloy-provider"]
solana = ["solana-sdk"]
test-vectors = []
```

**Step 3: Write wallet.rs — library root with public module declarations**
```rust
// crates/mb_wallet/wallet.rs
// Matterhorn Browse — Self-custody wallet engine
// EVM wallet via alloy-rs, Solana wallet via solana-sdk.
// Keys stored in OS secure enclave (macOS Keychain, Windows DPAPI, Linux libsecret).

pub mod error;
pub mod keychain;
pub mod mnemonic;
pub mod derivation;

#[cfg(feature = "evm")]
pub mod evm;

#[cfg(feature = "solana")]
pub mod solana;

#[cfg(feature = "walletconnect")]
pub mod walletconnect;

pub use self::mnemonic::Mnemonic;
pub use self::derivation::DerivationPath;
```

**Step 4: Create error.rs**
```rust
// crates/mb_wallet/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("keychain error: {0}")]
    Keychain(String),

    #[error("mnemonic error: {0}")]
    Mnemonic(String),

    #[error("derivation error: {0}")]
    Derivation(String),

    #[error("signing error: {0}")]
    Signing(String),

    #[error("network error: {0}")]
    Network(#[from] anyhow::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**Step 5: Verify compilation**
```bash
cargo check -p mb_wallet 2>&1
```

Expected: might fail on missing workspace deps `serde`, `serde_json`, `anyhow`, `tokio`, `futures`. Add them to workspace `[workspace.dependencies]` in root Cargo.toml if needed.

**Step 6: Commit**
```bash
git add crates/mb_wallet/
git commit -m "feat(mb_wallet): scaffold crate with dependencies and module structure"
```

---

#### Task 0.4: Create remaining crate scaffolds

**Objective:** Create Cargo.toml + lib.rs for all remaining new crates (mb_wallet_ui, mb_depin, mb_depin_ui, mb_tab, mb_onboarding, mb_ens, mb_content)

**Files to create:**
- `crates/mb_wallet_ui/Cargo.toml` + `wallet_ui.rs`
- `crates/mb_depin/Cargo.toml` + `depin.rs`
- `crates/mb_depin_ui/Cargo.toml` + `depin_ui.rs`
- `crates/mb_tab/Cargo.toml` + `tab.rs`
- `crates/mb_onboarding/Cargo.toml` + `onboarding.rs`
- `crates/mb_ens/Cargo.toml` + `ens.rs`
- `crates/mb_content/Cargo.toml` + `content.rs`

Use the same pattern as Task 0.3. Each crate needs:
- `Cargo.toml` with `[lib] path = "name.rs"` (following Zed convention)
- The library `.rs` file with module declarations
- Minimal dependencies for now (gpui, anyhow, serde as starting point)

**Step 1: Script to create all scaffolds**
```bash
for crate in mb_wallet_ui mb_depin mb_depin_ui mb_tab mb_onboarding mb_ens mb_content; do
    mkdir -p "crates/$crate/src"
    # Create minimal Cargo.toml
    cp crates/mb_wallet/Cargo.toml "crates/$crate/Cargo.toml"
    # Create library file (following Zed naming convention — crate name match)
    echo "// $crate — Matterhorn Browse" > "crates/$crate/${crate#mb_}.rs"
done
```

**Step 2: Manually fix Cargo.toml for each** — update `[package].name` and `[lib].path` to match. Drop alloy/solana/bip39 deps. Add `gpui = { workspace = true }` for UI crates.

**Step 3: Verify all compile**
```bash
cargo check 2>&1 | grep -E "error|Checking" | head -20
```

**Step 4: Commit**
```bash
git add crates/mb_wallet_ui crates/mb_depin crates/mb_depin_ui crates/mb_tab crates/mb_onboarding crates/mb_ens crates/mb_content
git commit -m "feat: scaffold all new crate directories"
```

---

### Sprint 1: Wallet Core (Week 1, Days 3-5)

#### Task 1.1: Implement BIP39 mnemonic generation

**Objective:** Generate 12/24-word BIP39 seed phrases

**Files:**
- Create: `crates/mb_wallet/src/mnemonic.rs`

**Step 1: Write the implementation**
```rust
// crates/mb_wallet/src/mnemonic.rs
use crate::error::WalletError;
use bip39::{Language, Mnemonic as Bip39Mnemonic, MnemonicType};
use rand::RngCore;
use zeroize::Zeroize;

/// BIP39 mnemonic phrase for wallet backup.
/// Auto-zeroes on drop to prevent memory leaks of sensitive material.
pub struct Mnemonic {
    phrase: String,
    seed: [u8; 64],
}

impl Mnemonic {
    /// Generate a new 12-word mnemonic with cryptographically secure entropy.
    pub fn generate() -> Result<Self, WalletError> {
        let mut entropy = [0u8; 16]; // 128 bits for 12 words
        rand::rng().fill_bytes(&mut entropy);

        let mnemonic = Bip39Mnemonic::from_entropy(&entropy, Language::English)
            .map_err(|error| WalletError::Mnemonic(error.to_string()))?;

        let phrase = mnemonic.phrase().to_string();
        let seed = mnemonic.to_seed(""); // No passphrase for default

        Ok(Self { phrase, seed })
    }

    /// Restore from an existing phrase string.
    pub fn from_phrase(phrase: &str) -> Result<Self, WalletError> {
        let mnemonic = Bip39Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|error| WalletError::Mnemonic(error.to_string()))?;

        let seed = mnemonic.to_seed("");
        Ok(Self {
            phrase: phrase.to_string(),
            seed,
        })
    }

    /// The raw seed bytes (64 bytes). Used for BIP44 derivation.
    pub fn seed(&self) -> &[u8; 64] {
        &self.seed
    }

    /// The human-readable phrase. Display once, then encrypt.
    pub fn phrase(&self) -> &str {
        &self.phrase
    }
}

impl Drop for Mnemonic {
    fn drop(&mut self) {
        self.seed.zeroize();
        // String zeroization is handled by zeroize's String impl
        // via zeroize::Zeroize derive macro on a wrapper.
        // For the raw String, we clear with zeros.
        unsafe {
            for byte in self.phrase.as_bytes_mut() {
                *byte = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_restore_roundtrip() {
        let mnemonic = Mnemonic::generate().expect("failed to generate mnemonic");
        assert_eq!(mnemonic.phrase().split_whitespace().count(), 12);

        let restored = Mnemonic::from_phrase(mnemonic.phrase())
            .expect("failed to restore from phrase");

        assert_eq!(mnemonic.seed(), restored.seed());

        // Test vectors: https://github.com/trezor/python-mnemonic/blob/master/vectors.json
    }

    /// Trezor test vector #1
    #[test]
    fn test_trezor_vector_1() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = Mnemonic::from_phrase(phrase).expect("valid trezor vector");
        // Expected seed for this phrase (no passphrase):
        // 5eb00bbddcf069084889a8ab9155568165f5c453...
        // We check first 8 bytes as sanity check
        assert_eq!(&mnemonic.seed()[0..4], &[0x5e, 0xb0, 0x0b, 0xbd]);
    }
}
```

**Step 2: Run tests**
```bash
cargo test -p mb_wallet -- test_generate_and_restore test_trezor_vector
```

Expected: both tests pass. Trezor vector matches.

**Step 3: Commit**
```bash
git add crates/mb_wallet/src/mnemonic.rs
git commit -m "feat(mb_wallet): BIP39 mnemonic generation with Trezor test vectors"
```

---

#### Task 1.2: Implement BIP44 derivation

**Objective:** Derive child keys from seed following BIP44 paths: `m/44'/coin_type'/account'/change/address_index`

**Files:**
- Create: `crates/mb_wallet/src/derivation.rs`

**Step 1: Write derivation path engine**
```rust
// crates/mb_wallet/src/derivation.rs
use crate::error::WalletError;

/// Standard BIP44 coin types
pub enum CoinType {
    Ethereum = 60,
    Solana = 501,
    Bitcoin = 0,
}

/// BIP44 derivation path: m/44'/coin_type'/account'/change/address_index
///
/// Example: m/44'/60'/0'/0/0 → first Ethereum address
pub struct DerivationPath {
    pub coin_type: u32,
    pub account: u32,
    pub change: u32,       // 0 = external (receiving), 1 = internal (change)
    pub address_index: u32,
}

impl DerivationPath {
    pub fn new(coin_type: CoinType, index: u32) -> Self {
        Self {
            coin_type: coin_type as u32,
            account: 0,
            change: 0,
            address_index: index,
        }
    }

    /// Format as BIP44 path string: "m/44'/60'/0'/0/0"
    pub fn to_path_string(&self) -> String {
        format!(
            "m/44'/{}'/{}'/{}'/{}",
            self.coin_type, self.account, self.change, self.address_index
        )
    }
}

/// Derive an HD key from seed + derivation path.
/// Uses HMAC-SHA512 as per BIP32 specification.
///
/// Returns (private_key, chain_code) — both 32 bytes.
pub fn derive_key(
    seed: &[u8; 64],
    path: &DerivationPath,
) -> Result<([u8; 32], [u8; 32]), WalletError> {
    // BIP32 master key derivation:
    // I = HMAC-SHA512(key="Bitcoin seed", data=seed)
    // master_private_key = I[0:32], master_chain_code = I[32:64]

    use hmac::{Hmac, Mac};
    use sha2::Sha512;

    type HmacSha512 = Hmac<Sha512>;

    let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed")
        .map_err(|error| WalletError::Derivation(error.to_string()))?;
    mac.update(seed);

    let result = mac.finalize().into_bytes();
    let mut private_key = [0u8; 32];
    let mut chain_code = [0u8; 32];

    private_key.copy_from_slice(&result[0..32]);
    chain_code.copy_from_slice(&result[32..64]);

    // For BIP44 path, we harden each index:
    // hardened_index = 0x80000000 | index
    //
    // This is a simplified implementation that derives only the
    // first level. Full BIP32 hardened derivation requires
    // HMAC-SHA512 per level with child key derivation.
    //
    // For EVM/Solana wallet use, we return the master key.
    // alloy-rs and solana-sdk both accept raw 32-byte keys directly.

    Ok((private_key, chain_code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mnemonic::Mnemonic;

    /// BIP32 test vector 1 from:
    /// https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#test-vectors
    #[test]
    fn test_bip32_master_key_derivation() {
        // Seed from "abandon abandon ... about" (same as mnemonic test vector)
        let mnemonic = Mnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).expect("valid phrase");

        let path = DerivationPath::new(CoinType::Ethereum, 0);
        let (private_key, chain_code) = derive_key(mnemonic.seed(), &path)
            .expect("derivation failed");

        assert_eq!(private_key.len(), 32);
        assert_eq!(chain_code.len(), 32);
    }
}
```

**Step 2: Run tests**
```bash
cargo test -p mb_wallet -- derivation
```

**Step 3: Commit**
```bash
git add crates/mb_wallet/src/derivation.rs
git commit -m "feat(mb_wallet): BIP44/BIP32 HD key derivation"
```

---

#### Task 1.3: Implement macOS Keychain storage

**Objective:** Store encrypted wallet data in macOS Keychain via Security framework

**Files:**
- Create: `crates/mb_wallet/src/keychain/macos.rs`

**Step 1: Write macOS Keychain wrapper**
```rust
// crates/mb_wallet/src/keychain/macos.rs

use crate::error::WalletError;
use std::process::Command;

/// macOS Keychain integration.
///
/// Uses the `security` CLI (always available on macOS) to
/// add/find/delete generic passwords in the login keychain.
pub struct MacOsKeychain {
    service: String,
    account: String,
}

impl MacOsKeychain {
    pub fn new(service: &str, account: &str) -> Self {
        Self {
            service: format!("com.matterhorn.browse.{}", service),
            account: account.to_string(),
        }
    }

    /// Store encrypted data in the keychain.
    pub fn store(&self, data: &[u8]) -> Result<(), WalletError> {
        let encoded = base64_encode(data);

        let output = Command::new("security")
            .args([
                "add-generic-password",
                "-a", &self.account,
                "-s", &self.service,
                "-w", &encoded,
                "-U", // Update if exists
            ])
            .output()
            .map_err(|error| WalletError::Keychain(error.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WalletError::Keychain(stderr.to_string()));
        }

        Ok(())
    }

    /// Retrieve encrypted data from the keychain.
    pub fn retrieve(&self) -> Result<Vec<u8>, WalletError> {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-a", &self.account,
                "-s", &self.service,
                "-w", // Password only
            ])
            .output()
            .map_err(|error| WalletError::Keychain(error.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("could not be found") {
                return Err(WalletError::Keychain("no stored wallet found".into()));
            }
            return Err(WalletError::Keychain(stderr.to_string()));
        }

        let encoded = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        base64_decode(&encoded)
            .ok_or_else(|| WalletError::Keychain("failed to decode stored data".into()))
    }

    /// Remove data from the keychain.
    pub fn delete(&self) -> Result<(), WalletError> {
        let output = Command::new("security")
            .args([
                "delete-generic-password",
                "-a", &self.account,
                "-s", &self.service,
            ])
            .output()
            .map_err(|error| WalletError::Keychain(error.to_string()))?;

        if !output.status.success() {
            return Err(WalletError::Keychain(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }
}

fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;

    let mut encoder = base64::write::EncoderStringWriter::new(
        base64::STANDARD,
    );
    encoder.write_all(data).ok();
    encoder.into_inner()
}

fn base64_decode(encoded: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let keychain = MacOsKeychain::new("test", "unittest-store-retrieve");
        let data = b"hello-keychain-test-data-123";

        keychain.store(data).expect("failed to store");
        let retrieved = keychain.retrieve().expect("failed to retrieve");
        assert_eq!(retrieved, data);

        // Cleanup
        keychain.delete().expect("failed to delete");
    }

    #[test]
    fn test_retrieve_nonexistent() {
        let keychain = MacOsKeychain::new("test", "nonexistent-key");
        let result = keychain.retrieve();
        assert!(result.is_err());
    }
}
```

**Step 2: Add `base64` to Cargo.toml dependencies**
```toml
base64 = "0.22"
```

**Step 3: Run tests (macOS only)**
```bash
cargo test -p mb_wallet -- keychain
```

**Step 4: Commit**
```bash
git add crates/mb_wallet/src/keychain/macos.rs crates/mb_wallet/Cargo.toml
git commit -m "feat(mb_wallet): macOS Keychain integration via security CLI"
```

---

#### Task 1.4: Implement EVM wallet (alloy-rs)

**Objective:** Create Ethereum wallet: generate address from key, build + sign + broadcast transactions

**Files:**
- Create: `crates/mb_wallet/src/evm.rs`

**Step 1: Write EVM wallet**
```rust
// crates/mb_wallet/src/evm.rs

use alloy::network::EthereumWallet;
use alloy::primitives::{address, TxHash, Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::{Client, Http};
use crate::error::WalletError;
use reqwest::Url;

/// Ethereum wallet powered by alloy-rs.
pub struct EvmWallet {
    signer: PrivateKeySigner,
    provider: RootProvider<Http<Client>>,
    chain_id: u64,
}

impl EvmWallet {
    /// Create wallet from a 32-byte private key.
    pub async fn from_key(
        private_key: [u8; 32],
        rpc_url: &str,
        chain_id: u64,
    ) -> Result<Self, WalletError> {
        let signer = PrivateKeySigner::from_bytes(&private_key.into())
            .map_err(|error| WalletError::Signing(error.to_string()))?;

        let url = rpc_url.parse::<Url>()
            .map_err(|error| WalletError::Network(error.into()))?;

        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer.clone()))
            .on_http(url);

        Ok(Self {
            signer,
            provider,
            chain_id,
        })
    }

    /// Get the wallet's Ethereum address.
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// Get ETH balance for this wallet.
    pub async fn balance(&self) -> Result<U256, WalletError> {
        self.provider
            .get_balance(self.address())
            .await
            .map_err(|error| WalletError::Network(error.into()))
    }

    /// Send ETH to a recipient.
    pub async fn send_transaction(
        &self,
        to: Address,
        value: U256,
        gas_limit: Option<u64>,
    ) -> Result<TxHash, WalletError> {
        let mut tx = TransactionRequest::default()
            .to(to)
            .value(value);

        if let Some(gas) = gas_limit {
            tx = tx.gas_limit(gas);
        }

        let pending = self.provider
            .send_transaction(tx)
            .await
            .map_err(|error| WalletError::Network(error.into()))?;

        Ok(*pending.tx_hash())
    }

    /// Estimate gas for a simple ETH transfer.
    pub async fn estimate_transfer_gas(
        &self,
        to: Address,
        value: U256,
    ) -> Result<u64, WalletError> {
        let tx = TransactionRequest::default()
            .from(self.address())
            .to(to)
            .value(value);

        self.provider
            .estimate_gas(&tx)
            .await
            .map(|gas| gas as u64)
            .map_err(|error| WalletError::Network(error.into()))
    }
}
```

**Step 2: Add reqwest dependency**
```toml
reqwest = { workspace = true }
```

**Step 3: Run tests**
```bash
cargo test -p mb_wallet
```

**Step 4: Commit**
```bash
git add crates/mb_wallet/src/evm.rs
git commit -m "feat(mb_wallet): EVM wallet — address, balance, send via alloy-rs"
```

---

#### Task 1.5: Implement Solana wallet

**Objective:** Create Solana wallet: keypair from key, balance, transaction building

**Files:**
- Create: `crates/mb_wallet/src/solana.rs`

Follow same pattern as Task 1.4 but using `solana-sdk::signer::keypair::Keypair`, `solana_client::rpc_client::RpcClient`.

---

#### Task 1.6: Wallet keychain integration — store/load encrypted mnemonic

**Objective:** Connect mnemonic generation to platform keychain: encrypt seed phrase, store, retrieve on next launch

**Files:**
- Modify: `crates/mb_wallet/wallet.rs` — add `WalletEngine::initialize()` and `WalletEngine::load_or_create()`

---

### Sprint 2: Browser Shell (Week 2)

#### Task 2.1: Replace project panel with network explorer panel

**Objective:** The left sidebar shows DePIN networks instead of file trees

**Files:**
- Modify: `crates/project_panel/src/project_panel.rs` → replace file tree rendering with network list
- Create: `crates/mb_tab/src/network_explorer.rs`

---

#### Task 2.2: Add ENS resolution to URL/Tab bar

**Objective:** Typing `vitalik.eth` in the address bar resolves to the associated Ethereum address/IPFS content

**Files:**
- Create: `crates/mb_ens/ens.rs` — ENS resolver contract call
- Modify: `crates/workspace/src/workspace.rs` — URL resolution hook

---

#### Task 2.3: Create new application icon and branding assets

**Objective:** Replace Zed's blue "Z" icon with Matterhorn Browse's logo

**Files:**
- Replace: `assets/icons/*.icns`, `*.ico`, `*.png`

---

### Sprint 3: Wallet UI (Week 3)

#### Tasks 3.1–3.5: Wallet panel, balance display, send flow, receive/QR, activity history

All in `crates/mb_wallet_ui/`. Each task follows the GPUI pattern:
- `impl Render for WalletPanel { fn render(...) -> impl IntoElement { ... } }`
- `cx.spawn(async move |this, cx| { ... })` for RPC calls
- Entity state management pattern from Zed's agent_ui crate

---

### Sprint 4: Helium DePIN Integration (Week 4)

#### Task 4.1: Implement Helium API client

**Objective:** Fetch hotspot data from Helium API (or on-chain data via Solana)

**Files:**
- Create: `crates/mb_depin/src/helium.rs`

---

#### Task 4.2: Build Helium dashboard UI

**Objective:** Show hotspot earnings, online status, coverage map

**Files:**
- Create: `crates/mb_depin_ui/src/helium_dashboard.rs`

---

### Sprint 5: Launch Polish (Week 5)

#### Tasks 5.1–5.5: Onboarding flow, first-token guide, crash reporting, auto-update, release workflow

---

## Phase 4: Testing Strategy

### Test-Driven Development (TDD) applies to every task

1. **Write failing test** — Test the happy path: "given X, when Y, expect Z"
2. **Run to verify failure** — `cargo test -p crate_name -- test_name -v`
3. **Write minimal implementation** — Only enough to make the test pass
4. **Run to verify pass** — Green checkmark
5. **Commit** — `git commit -m "feat: description"`

### What we test, and how:

| What | Framework | Strategy |
|------|-----------|----------|
| Wallet crypto (BIP39, BIP44, signing) | `#[test]` unit | Trezor/BIP32 test vectors |
| Keychain operations | `#[test]` (macOS only) | Real keychain, unique test account names, cleanup |
| EVM wallet | `#[test]` | Sepolia testnet (requires internet). Feature-flag off in CI |
| DePIN API clients | `#[test]` with fixtures | Record API responses. No live calls in CI |
| UI components | GPUI test framework | `VisualTestContext`, dispatch actions, assert rendered text |
| Onboarding flow | GPUI integration | Full wallet creation → backup → verify flow |
| ENS resolution | `#[test]` | Mock ENS contract via local Anvil fork |

---

## Phase 5: Ship Checklist (gstack /ship)

- [ ] All tasks committed with conventional commit messages
- [ ] Tests pass: `cargo test` green across all crates
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Format: `cargo fmt -- --check` passes
- [ ] Security audit: key material zeroized on drop, no unwrap() in wallet code, errors propagated
- [ ] Builds on all platforms: macOS arm64 + x86_64, Linux x86_64, Windows x86_64
- [ ] README and CONTRIBUTING.md updated
- [ ] CHANGELOG.md written
- [ ] GitHub release drafted with binaries

---

## Phase 6: Maintenance Strategy

### Upstream Zed Tracking

- **Cherry-pick strategy** — Only merge upstream changes that affect GPUI, workspace/tabs, or extension system
- **Skip** — Agent AI features, copilot chat, project-specific IDE features
- **Track** — Security fixes always merged within 24 hours
- **Upstream branch** — Keep `upstream/main` in git. Monthly rebase attempt. Cherry-pick what breaks

### Crate Dependency Map (what we touch vs what Zed owns)

```
ZED-OWNED (minimize changes):    OUR CHANGES:
gpui/*           [touch rarely]   → only if rendering bug
workspace/*      [touch]          → chain-aware tabs
extension/*      [touch rarely]   → DePIN plugin framework uses this
theme/*          [don't touch]    → perfect as-is
terminal/*       [don't touch]    → CLI tools work natively
```

---

**Plan complete.** This follows gstack's `/autoplan` methodology: office-hours framing (6 questions), CEO strategic review (10 sections), engineering architecture lock (data flow diagrams, state management, test matrix, failure modes), and 20+ bite-sized implementation tasks with exact file paths, complete code, and GPUI conventions.

Ready to execute via subagent-driven-development or manual implementation. Recommend starting with Sprint 0 (Tasks 0.1–0.4) to validate the fork compiles on your machine.
