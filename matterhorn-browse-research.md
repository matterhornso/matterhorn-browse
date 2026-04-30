# Matterhorn Browse — Market Research & Competitive Landscape

*Compiled: April 30, 2026 | For: Matterhorn 2.0 Launch Strategy*

---

## 1. THE GAP: What's Missing

### The landscape

There are three product categories adjacent to "Matterhorn Browse." None of them solve the whole problem:

| Category | Examples | What they do | What they DON'T do |
|----------|---------|---------------|---------------------|
| **Web3 browsers** | Brave, Opera Crypto | Wallet in a browser, some dapp discovery | No DePIN integration, no explorer/IDE hybrid, no self-sovereign data UX |
| **Wallets-as-platforms** | Phantom, MetaMask, Rainbow | Browser extension for signing + dapp connections | Not a browser. Tabbed browsing is an afterthought. No content rendering or exploration |
| **Block explorers** | Etherscan, Solscan | Read-only chain data | Not a browser. No wallet. No interaction layer beyond viewing |

**The gap:** There is no application that combines a *browser* (web rendering + tabs + content consumption), a *wallet* (keys + signing + balances), and a *DePIN dashboard* (network stats + device management + earnings tracking) into one native desktop experience.

---

## 2. COMPETITOR DEEP DIVES

### 2.1 Brave Browser

- **What it is:** Chromium fork with built-in ad blocker, Tor tabs, and Brave Wallet (EVM + Solana)
- **Wallet:** Non-custodial, HD wallet, supports hardware wallets, NFT gallery, swap aggregator
- **DePIN:** Zero integration. You can use dapps via the wallet but there's no native DePIN view, no network management, no device earning dashboard
- **Web3 discovery:** Brave News and Brave Search are privacy-focused, not Web3-focused. There's no "explore Web3" layer
- **Limitations:** Chromium bloat (~400MB RAM idle). The wallet is a sidebar feature, not the core UX. DePIN users have no reason to choose Brave over Chrome + MetaMask
- **User base:** ~70M MAU (as of late 2025). Primarily privacy-conscious users, NOT Web3 natives
- **Takeaway for Matterhorn:** Brave proves wallets-in-browsers work. But Brave is a privacy browser with wallet bolted on. Matterhorn Browse can be a Web3 browser from the ground up

### 2.2 Opera Crypto Browser

- **What it is:** Opera fork with built-in crypto wallet, Web3 domain resolution (.eth, .crypto, etc.)
- **Wallet:** Multi-chain (EVM, Bitcoin, Solana), built-in swap, dapp catalog, clipboard security
- **DePIN:** None
- **Limitations:** The "Crypto Browser" was mostly a skinned Opera. Opera pivoted away from it in 2024–2025 after failing to gain traction. The project has been largely absorbed back into mainline Opera with crypto features becoming just a toggle
- **Takeaway:** The "crypto browser" as a separate product failed when it was just a skin. The integration needs to be deep and native, not a coat of paint

### 2.3 Osiris Browser

- **What it is:** Chromium-based browser with built-in Metawallet (self-custody), dapp store, decentralized DNS
- **Wallet:** Self-custody, supports ETH, BSC, Polygon. Integrated dapp store with curated listings
- **DePIN:** None
- **Limitations:** Small team, limited adoption. Feels like a reskinned Chromium. The "dapp store" is basically a bookmark folder
- **Takeaway:** Self-custody wallets in browsers are table stakes now

### 2.4 Puma Browser

- **What it is:** Mobile-only Web3 browser. Built-in Interledger Protocol (ILP) for micropayments
- **Wallet:** ILP-native wallet for streaming micropayments to creators
- **DePIN:** None
- **Limitations:** Mobile-only. ILP adoption is nearly zero. Niche beyond niche
- **Takeaway:** Interesting payments vision, but the Web3 world went DeFi + DePIN, not ILP

### 2.5 Code Editors with Web3 (adjacent)

- **VS Code + extensions:** Solidity/Vyper/Move extensions exist. MetaMask and other wallet extensions. But it's a code editor, not a browser
- **Remix IDE:** Browser-based Solidity IDE. Has wallet connections. But it's for writing smart contracts, not browsing the web
- **Foundry/Hardhat:** CLI-based dev tooling. No UI layer at all
- **Zed itself:** GPU-accelerated IDE in Rust. GPUI is a real-time collaborative rendering engine. Extensible via extensions. No Web3 features today
- **Takeaway:** No code editor has attempted to be a Web3 browser. The IDE + browser fusion is genuinely novel

### 2.6 Other Mentions

- **Beaker Browser:** P2P Hyperdrive protocol browser. Shut down 2022. Interesting model but predates Web3
- **Agregore:** IPFS-based browser. Developer tool, not consumer-facing
- **Mises Browser:** Web3 mobile browser focused on Cosmos ecosystem. Small adoption
- **Carbon Browser:** Privacy-focused mobile browser with crypto rewards. Ad-block + VPN + token rewards model. Decent traction in developing markets

---

## 3. DEPIN LANDSCAPE (April 2026)

### 3.1 Market Size

- **Messari estimates (Jan 2025):** DePIN sector market cap crossed **$50B**. Annualized on-chain revenue exceeding **$500M** across all DePIN protocols
- **Helium alone:** 1M+ hotspots deployed, $100M+ annual DC burn
- **Render Network:** 5M+ frames rendered on-chain in 2025
- **io.net:** 300K+ GPUs on the network as of Q1 2026
- **Filecoin:** 25+ EiB storage capacity, 3,000+ storage providers
- **Growth projections:** The sector is growing ~200% YoY in both users and revenue. By 2027, DePIN is projected to be a $200B+ market cap sector

### 3.2 Major DePIN Categories & Networks

#### Compute
| Network | What it does | User interface today | What a browser could add |
|---------|-------------|---------------------|-------------------------|
| **Render Network** | Decentralized GPU rendering | Web dashboard, CLI tools | Native render job submission, progress tracking, earnings dashboard |
| **Akash Network** | Decentralized cloud compute | CLI (`akash`), Cloudmos web UI | One-click deploy from browser, resource monitoring, cost comparison vs AWS/GCP |
| **io.net** | GPU clusters for AI training | Web dashboard, REST API | GPU availability browser, job scheduling, performance benchmarks |
| **Golem Network** | Distributed computing | Desktop app (Electron) + CLI | Replace Electron app with native GPUI experience. Task marketplace browsing |

#### Storage
| Network | What it does | User interface today | What a browser could add |
|---------|-------------|---------------------|-------------------------|
| **Filecoin** | Decentralized storage | Lotus CLI, web-based explorers, Estuary | Drag-and-drop file upload, storage deal browser, retrieval speed testing, provider reputation browser |
| **Arweave** | Permanent storage | Web app (arweave.app), CLI | Permaweb browsing as first-class citizen. Archive creation. Content addressing built into URL bar |
| **Storj** | S3-compatible decentralized storage | Web dashboard, S3 SDK | S3 bucket management, usage dashboards, earnings tracker for node operators |

#### Wireless / IoT
| Network | What it does | User interface today | What a browser could add |
|---------|-------------|---------------------|-------------------------|
| **Helium** | Decentralized wireless (IoT 5G, Mobile) | Helium app (mobile), Explorer website | Hotspot management dashboard, earnings tracking, coverage mapping, device onboarding |
| **Helium Mobile** | Decentralized cellular | Mobile app | Usage stats, data plan management, coverage explorer |
| **DIMO** | Connected vehicle data | Mobile app | Vehicle data dashboard, marketplace browsing, earnings tracking |

#### Mapping / Geospatial
| Network | What it does | User interface today | What a browser could add |
|---------|-------------|---------------------|-------------------------|
| **Hivemapper** | Decentralized mapping | Mobile app, web dashboard | Map explorer, coverage gaps browser, dashcam contribution stats, token earnings |

#### Energy
| Network | What it does | User interface today | What a browser could add |
|---------|-------------|---------------------|-------------------------|
| **Daylight** | Decentralized energy | Mobile app | Energy production dashboard, grid contribution tracking |
| **Powerledger** | Energy trading | Web platform | Energy marketplace browsing, P2P trade execution |

### 3.3 The DePIN UX Problem

Every DePIN network has its own:
- Dashboard (different URL, different login)
- CLI tool (different commands, different install)
- Token (different wallet integration)
- Explorer (different UI paradigm)

**No one has built a unified DePIN browser.** A single application that:
1. Shows your assets across all DePIN networks
2. Manages your devices (hotspots, dashcams, GPUs, storage nodes)
3. Tracks your earnings in one dashboard
4. Lets you deploy workloads / buy storage / contribute bandwidth from one place
5. Browses the on-chain state of any DePIN network natively

This is the core opportunity for Matterhorn Browse.

---

## 4. WALLET SDK & EMBEDDING ANALYSIS

### 4.1 The Key Decision: WalletConnect vs Self-Custody vs Embedded

Three architectural approaches for putting a wallet in a desktop app:

#### Approach A: Connect to External Wallet (WalletConnect v2)
- **How:** User connects their existing MetaMask/Phantom/Rainbow to Matterhorn Browse via WalletConnect relay
- **Chains:** EVM, Solana, Cosmos (depends on wallet)
- **Rust binding:** No official WalletConnect Rust SDK. Would need to implement the WalletConnect v2 protocol (WebSocket relay + JSON-RPC over encrypted channel) manually. This is ~2-3 weeks of Rust work
- **UX:** User needs an existing wallet. Friction for newcomers. Good for experienced Web3 users
- **Risk:** Relies on WalletConnect relay infrastructure (centralized). If relay is down, no wallet connectivity

#### Approach B: Native Self-Custody (In-App Keys)
- **How:** Matterhorn Browse generates and stores private keys locally. Full control
- **Tech options:**
  - **alloy-rs** (Ethereum): Successor to ethers-rs. Modern, fast, well-maintained Rust library for EVM interaction. Supports signing, ABI encoding, providers, contract interaction
  - **solana-sdk** (Solana): Native Rust SDK. Full support for keypair generation, transaction building, program interaction
  - **cosmrs** (Cosmos): Rust implementation of Cosmos SDK
- **UX:** No external wallet needed. Can be a user's first wallet. Best onboarding for "anyone curious about Web3"
- **Risk:** Key management is hard. Need secure enclave (macOS Keychain / Windows DPAPI). Phishing risks if browser renders malicious dapps

#### Approach C: Embedded Wallet-as-a-Service
- **How:** Use a WaaS provider (Privy, Dynamic, Turnkey) to handle key management
- **Tech:** These services provide SDKs (React/JS). No desktop Rust SDKs exist today. Would need a bridge via embedded web view or Tauri-style IPC
- **UX:** Email/social login → wallet created behind the scenes. Smooth onboarding
- **Risk:** Dependency on third-party infra. Pricing at scale. Limited customization

### 4.2 Recommended Approach for Matterhorn Browse

**Hybrid: Approach B (native self-custody) + Approach A (WalletConnect fallback)**

- **Primary:** In-app self-custody wallet using `alloy-rs` (EVM) + `solana-sdk` (Solana)
- **Fallback:** WalletConnect integration for users who already have wallets
- **Key storage:** Platform-native secure enclave (macOS Keychain via Security framework, Windows DPAPI via winapi, Linux via freedesktop secrets)
- **Mnemonic:** BIP39 seed phrase, encrypted at rest, never leaves the device
- **HD derivation:** BIP44 for multi-chain address derivation from one seed

### 4.3 Rust Crate Landscape for Wallet

| Crate | Purpose | Maturity | Maintainer |
|-------|---------|----------|------------|
| `alloy` | EVM client: signing, providers, contract interaction, transports | Production-ready, actively maintained | alloy-rs (used by Reth, Foundry) |
| `alloy-signer` | Signing implementations (local, Ledger, Trezor, AWS KMS) | Production | alloy-rs |
| `solana-sdk` | Full Solana program interaction, transaction building, keypair management | Production | Solana Labs / Anza |
| `solana-client` | RPC client for Solana | Production | Solana Labs / Anza |
| `cosmrs` | Cosmos SDK in Rust | Active development | Informal Systems |
| `bitcoin` | Rust Bitcoin library (PSBTs, descriptors, Taproot) | Production | rust-bitcoin community |
| `bip39` / `bip32` | Mnemonic generation + HD key derivation | Stable (though some unmaintained) | Community |
| `zeroize` | Secure memory zeroing for sensitive key material | Stable | RustCrypto |

---

## 5. MARKET POSITIONING: Matterhorn Browse

### 5.1 The Target: "Anyone curious about Web3"

This is a bigger market than "developers" or "degens." It includes:

1. **The Curious Newcomer** — Heard about Web3, doesn't know where to start. Wants to browse, not code. Needs an approachable, beautiful entry point
2. **The DePIN Operator** — Running a Helium hotspot or a Hivemapper dashcam. Wants to monitor earnings, manage devices, explore new network opportunities
3. **The dApp Explorer** — Uses DeFi, NFTs, or Web3 social. Wants a cleaner, faster, more native experience than browser+extension
4. **The Content Consumer** — Reads Web3 news, follows protocols on-chain, checks ENS profiles. Wants a browsing experience that treats on-chain data as first-class content

### 5.2 Positioning Statement

> **Matterhorn Browse is the Web3-native browser.** Unlike Brave (a privacy browser with a wallet) or MetaMask (a wallet extension in a Chrome tab), Matterhorn Browse treats Web3 as a first-class citizen — every URL resolves on-chain, every page can be signed, every DePIN network is a tab away.

### 5.3 Brand Alignment (Matterhorn Voice)

Matterhorn Browse fits the Matterhorn family:
- **Matterhorn 2.0** (Cowork for Web3) — Build, collaborate, work
- **Matterhorn Browse** — Explore, discover, consume

Tone: Direct, technical, no fluff. Confident not arrogant. Builder-first.
Tagline candidate: *"Browse Web3. Natively."*

### 5.4 Competitive Moat (Why this isn't easy to copy)

1. **GPUI rendering engine** — GPU-accelerated, 5x lighter than Chrome. This isn't Electron. This is a real technical moat
2. **Native wallet** — Not a browser extension. Wallet as OS-level integration (keychain, secure enclave)
3. **Unified DePIN layer** — No one has built this. It's cross-network infra that compounds in value as more networks are added
4. **Rust performance** — The entire stack (rendering, networking, wallet crypto, DePIN SDKs) is in Rust. No JavaScript bridge overhead for signing or chain queries

---

## 6. ROUGH ARCHITECTURAL MAP (for planning)

### What stays from Zed
- **GPUI** — Rendering engine, compositing, text rendering, theming
- **Workspace/Tabs** — Multi-tab management, split panes
- **Collab** — Real-time collaboration primitives (could power shared DePIN dashboards)
- **Extension system** — Plugin framework for community DePIN integrations
- **Terminal** — Embedded PTY (useful for CLI-based DePIN tools)

### What gets removed/stripped
- **Agent AI panel** — Keep optional, not core
- **Project panel** — Replace with "Network explorer" (DePIN networks instead of file trees)
- **Code editing (full)** — Keep a lightweight editor panel for config files + smart contract viewing
- **LSP integration** — Keep for Solidity/Vyper, drop for non-Web3 languages
- **Git integration** — Drop entirely
- **Debugger** — Keep for WASM/on-chain debugging

### What's new (Matterhorn Browse crates)

```
crates/
  mb_wallet/           # Self-custody wallet engine
    ├── keychain/      # Platform-native key storage (macOS Keychain, Windows DPAPI, Linux)
    ├── evm/           # EVM wallet (alloy-rs based)
    ├── solana/        # Solana wallet
    ├── walletconnect/ # WalletConnect v2 relay client
    └── ui/            # Wallet panel UI (balances, activity, asset list)
  mb_depin/            # DePIN integration layer
    ├── helium/        # Helium hotspot management
    ├── render/        # Render Network job browser
    ├── akash/         # Akash deployment viewer
    ├── filecoin/      # Filecoin storage browser
    ├── iotex/         # IoTeX device management (Pebble tracker, UCam)
    └── framework/     # Plugin framework for community DePIN integrations
  mb_tab/              # Enhanced tab with chain-awareness
    ├── dapp_tab/      # Dapp tab (isolated context, injected provider)
    ├── explorer_tab/  # Block explorer tab (parses chain data natively)
    └── depin_tab/     # DePIN network dashboard tab
  mb_onboarding/       # Web3 onboarding flow (wallet creation, key backup, first token)
  mb_ens/              # ENS / decentralized DNS resolution in URL bar
  mb_content/          # Content resolver (.ipfs, .arweave, .lens, .farcaster)
```

---

## 7. GO-TO-MARKET CONSIDERATIONS

### 7.1 Distribution
- **GitHub releases** (open source)
- **Homebrew** (macOS) / **winget** (Windows)
- **Matterhorn 2.0 in-app marketplace** — Browse is a "skill" or "workspace" inside Matterhorn 2.0
- **DePIN community directories** — List in Helium, Render, Filecoin community hubs

### 7.2 Launch Sequencing
1. **Phase 1 (Alpha):** Fork Zed, strip to browser shell, add basic EVM wallet, ENS resolution, web rendering
2. **Phase 2 (Beta):** Add Solana wallet, WalletConnect, first 3 DePIN integrations (Helium, Render, Filecoin)
3. **Phase 3 (Launch):** DePIN plugin framework, dapp isolation, mobile parity via PWA

### 7.3 Open Questions
- License: Zed is AGPL-3.0. The fork must remain AGPL-3.0. Acceptable for Matterhorn's open-source strategy?
- Maintenance: How to track upstream Zed releases? Cherry-pick or full rebase strategy?
- Branding: Does "Matterhorn Browse" fit alongside Matterhorn 2.0? Or is a different family name better?

---

## 8. KEY DIFFERENTIATORS (SUMMARY)

| Dimension | Brave/Opera | MetaMask/Phantom | Matterhorn Browse |
|-----------|------------|-------------------|-------------------|
| Rendering | Chromium (heavy) | Browser extension | GPUI (GPU-native, lightweight) |
| Wallet depth | Basic wallet | Deep wallet, no browser | Deep wallet + browser together |
| DePIN | None | None | Built-in multi-network DePIN dashboard |
| On-chain URL resolution | Limited (.eth via extension) | Via extension | Native ENS/IPFS/Arweave resolution |
| Performance | ~400MB idle | N/A (depends on host) | ~50MB idle (GPUI) |
| Target user | Privacy-focused general audience | Existing crypto users | Web3-curious + DePIN operators |
| Entry barrier | Download browser + configure wallet | Install Chrome + install extension | One download, wallet included |
| Open source | Mostly (MPL-2.0) | Partially | Fully (AGPL-3.0) |

---

## 9. RECOMMENDATIONS

1. **Build the browser, not the IDE.** Strip Zed's code editing features. Keep the rendering engine, tab system, and terminal. Add wallet + DePIN. This is a lighter fork with clearer product identity

2. **Start with self-custody EVM + Solana.** `alloy-rs` and `solana-sdk` are production-ready Rust crates. Build the wallet from these. Add WalletConnect as fallback later

3. **DePIN: start with Helium + Render.** Helium has the largest consumer-facing DePIN user base (hotspot owners). Render has the most "browser-applicable" use case (render job submission/viewing). These two establish the pattern

4. **Launch alongside Matterhorn 2.0.** May 2026. Position Browse as "the browser for the Matterhorn ecosystem" — the exploration layer while Matterhorn 2.0 is the build layer

5. **Create the DePIN browser category.** No one owns this term. Matterhorn Browse can define it

---

*Next: Implementation plan (separate document) — task breakdown, file paths, engineering estimates*
