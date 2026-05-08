# Matterhorn Browser — Implementation Spec

> **Status:** Build-ready  
> **Target launch:** May 12, 2026 (MVP — Layers 1–3) / Post-launch (Layers 4–5)  
> **Repo:** `https://github.com/matterhornso/zed` (fork of zed-industries/zed)  
> **Tagline:** "Browse Web3. Natively."

---

## Overview

Matterhorn Browser is a Web3-native desktop browser built on **GPUI** (Zed's GPU-accelerated UI framework). It unifies four currently disconnected Web3 touchpoints — AI search, wallet, dApp browser, and DePIN dashboards — into a single native desktop application.

Unlike Brave (Chromium + bolted-on wallet) or MetaMask (browser extension), Matterhorn is a browser whose rendering engine, wallet, and AI orchestration share a single Rust process. The browser _is_ the wallet. The browser _is_ the agent.

### The problem

Today a Web3 user's stack looks like:

```
[Perplexity/Dora for search] → [MetaMask/Phantom popup] → [Brave/Chrome tab] → [Helium/Render dashboard]
```

These tools don't share state, don't share context, and force the user to be the integration layer. Copy-pasting addresses, re-authenticating, manually cross-referencing. This is the fragmentation Matterhorn collapses.

### The answer

A single input surface. Type a URL, a natural-language question, a transaction intent, or a multi-step operation. The browser figures out what you meant and executes it — with wallet-native confirmation, not extension popups.

---

## Architecture: 5 Layers

```
┌──────────────────────────────────────────────────────────┐
│ L5  Execution & Rendering                                │
│     dApp viewport · AI sidebar · TX previews             │
├──────────────────────────────────────────────────────────┤
│ L4  DePIN AI Mesh                                        │
│     Search summarizers · Security analyzers · TX sims    │
├──────────────────────────────────────────────────────────┤
│ L3  Native Wallet & Identity Engine                      │
│     alloy-rs (EVM) · solana-sdk · Keychain/DPAPI         │
├──────────────────────────────────────────────────────────┤
│ L2  AI Orchestration Engine                              │
│     Intent parser · Router · Execution planner           │
├──────────────────────────────────────────────────────────┤
│ L1  Unified Composer Surface                             │
│     Single input bar · URL/NL/TX/multi-step              │
└──────────────────────────────────────────────────────────┘
```

---

### Layer 1 — Unified Composer Surface

The top-level UI: a single input bar replacing the traditional address bar.

**Accepts four input modes:**

| Mode | Example | Detection |
|------|---------|-----------|
| **URL** | `app.uniswap.org` or `matterhorn.so` | Standard URL pattern |
| **Natural language** | "find the best stablecoin yield on Arbitrum" | NL query (no URL pattern) |
| **Transaction intent** | "send 0.1 ETH to vitalik.eth" | Contains value + action + target |
| **Multi-step** | "bridge 500 USDC from Base to Arbitrum, then deposit into Aave" | Contains sequencing keywords (then, and, after) |

**UI requirements:**

- Rendered as a single text field spanning the top of the window
- Subtle mode indicator (icon/color hint) that updates live as the user types
- Command palette accessible via `Cmd+K` for power users
- Recent history / suggestions dropdown on focus
- Keyboard-first: `Enter` submits, `Tab` cycles suggestions

**Implementation notes:**

- Build as a new GPUI view (`composer::Composer`)
- Port Zed's existing command palette pattern for the dropdown
- Mode detection: start with simple regex heuristics; graduate to the L2 intent parser once built

---

### Layer 2 — AI Orchestration Engine

The intelligence layer. Takes raw input from L1 and determines what to do with it.

**Three components:**

#### 2a. Intent Parser
- Accepts raw text from the composer
- Classifies into: `navigate`, `search`, `transact`, `multi_step`, `unknown`
- Extracts entities: addresses, ENS names, token symbols, amounts, chain names, protocols
- Implementation: call an LLM with a structured-output prompt (JSON schema). Start with OpenAI-compatible API, make provider pluggable.

#### 2b. Router
- Maps parsed intent → execution plan
- `navigate` → render URL in dApp viewport (L5)
- `search` → query DePIN mesh (L4) or fallback search API
- `transact` → open wallet confirmation sheet (L3 → L5)
- `multi_step` → break into sequential sub-intents, present as a "plan card" for user approval

#### 2c. Execution Planner
- For `multi_step` intents: chains sub-intents with dependency awareness
- Renders a visual step-by-step plan before executing
- User can approve all, approve step-by-step, or cancel
- Each step produces a verifiable result (tx hash, balance diff, etc.)

**Implementation notes:**

- Crate: `matterhorn_orchestrator` (new)
- LLM integration: configurable endpoint + API key in settings. Use `reqwest` for HTTP calls.
- Structured output: enforce JSON schema on the LLM response. Parse into Rust enums.
- The orchestrator runs on a background thread; communicates with UI via GPUI's entity/notification system.

---

### Layer 3 — Native Wallet & Identity Engine

The wallet is not an extension. It is a core browser subsystem with direct access to the renderer.

**Design principles:**

- **No popups.** Transaction confirmation is an inline sheet rendered by GPUI, not a separate window.
- **Self-custody by default.** Keys generated on-device, stored in OS keychain.
- **Multi-chain from day one.** EVM (alloy-rs) + Solana (solana-sdk).

#### 3a. Key Management
- Generate BIP39 mnemonics, derive keys per chain (BIP44 paths)
- Store encrypted seed in macOS Keychain / Windows DPAPI via `security-framework` / `windows-rs`
- Optional: hardware wallet support via USB HID (post-MVP)
- Optional: WalletConnect v2 for pairing with mobile wallets (post-MVP)

#### 3b. Transaction Signing
- EVM: build + sign with `alloy-rs` (`alloy::providers`, `alloy::signers`)
- Solana: build + sign with `solana-sdk`, `solana-client` for RPC
- RPC endpoints: configurable per chain. Default to public endpoints; user can set own.

#### 3c. Identity
- ENS resolution (forward + reverse) via `alloy-rs` ENS support or dedicated crate
- Solana name service (`.sol` domains) via `sns-sdk` or direct RPC queries
- Address display: show ENS/SNS names, truncate raw addresses

#### 3d. Confirmation UI
- Inline sheet rendered in GPUI
- Shows: what (transfer / approve / contract call), to whom, amount, network, gas estimate
- Human-readable when possible (ENS names, token symbols, USD value)
- One-click confirm or reject
- Post-confirmation: overlay a small toast with tx hash + block explorer link

**Implementation notes:**

- Crate: `matterhorn_wallet` (new)
- Keychain access crate: `security-framework` (macOS), `windows-rs` credentials (Windows)
- The wallet holds an `Arc<Mutex<WalletState>>` shared with the orchestrator
- On first launch: onboarding flow → generate/import seed → encrypt → store in keychain
- Unlock on subsequent launches: prompt for password → decrypt seed from keychain

---

### Layer 4 — DePIN AI Mesh (Post-MVP)

A decentralized network of specialized AI nodes that the browser queries for intelligence.

**Node types:**

| Node type | Function | Example query |
|-----------|----------|---------------|
| Search summarizer | Web3-specific search with summarization | "what's the APY on Aave for USDC on Arbitrum?" |
| Security analyzer | Scan contract addresses, simulate transactions | "is this token safe to approve?" |
| TX simulator | Fork-simulate a transaction, return balance diff | "what happens if I swap 1 ETH for USDC?" |
| Price / data oracle | Real-time on-chain data with context | "gas prices across L2s right now" |

**MVP approach for DePIN (post-launch):**

- Build a simple **plugin interface** for third-party nodes
- Start with **two curated providers**: Helium (network data) and Render (GPU compute for AI models)
- Plugin protocol: JSON-RPC over WebSocket. Node operators register, browser discovers via registry.
- Token economics for node operators: deferred to post-MVP

**Implementation notes:**

- Crate: `matterhorn_depin` (new, post-MVP)
- Plugin system modeled on Zed's existing extension architecture
- For MVP, hardcode a direct integration with a single AI search endpoint
- The orchestrator queries DePIN nodes as one of its routing options

---

### Layer 5 — Execution & Rendering

The visual layer. What the user sees after the orchestrator decides what to do.

#### 5a. dApp Viewport
- Full web rendering surface for dApps
- **Question:** rendering engine — options:
  - **Option A:** Embed WebKit via `wry` or `webview` crate (Chromium/WebKit, heavy but full compatibility)
  - **Option B:** Build a minimal Web3 renderer (HTTP fetch + render JSON-RPC responses in GPUI)
  - **Recommendation:** Option A for MVP. A Web3-native renderer can come later.
- Injected context: the wallet's current account, chain ID, and a secure `window.matterhorn` object that dApps can use (similar to `window.ethereum`)

#### 5b. AI Sidebar
- Persistent right-hand panel that maintains context across tabs
- Shows: current context summary, recent actions, relevant on-chain data
- Can be toggled open/closed with `Cmd+B`
- The sidebar is the "memory" of the browsing session

#### 5c. Transaction Previews
- Before a dApp triggers a wallet action, the browser renders a human-readable preview
- Example: instead of "Approve 0x7a250... spend 1000000000000000000", show "Uniswap wants permission to spend 1 ETH"
- Uses the security analyzer from L4 (or a local simulation) when available

#### 5d. Tab Management
- Browser-style tabs at the top (reuse Zed's tab infrastructure)
- Each tab = a dApp viewport or a search result
- Pinned tabs for frequently-used dApps
- Tab groups for research sessions

**Implementation notes:**

- Crate: `matterhorn_viewport` (new, contains the webview embedding)
- Crate: `matterhorn_sidebar` (new, the AI context panel)
- Tab management: extend Zed's `workspace` crate with browser-tab semantics
- The `window.matterhorn` provider: implement as JavaScript injected into the webview on page load

---

## What to Strip from Zed

Zed has ~240 crates and ~1.3M lines of Rust. For Matterhorn Browser, we **keep** the rendering foundation and **remove** the IDE.

| Keep | Remove |
|------|--------|
| `gpui` — GPU UI framework | `editor` — text editor |
| `workspace` — tab/window management | `project` — project file tree |
| `terminal` — integrated terminal | `language` — LSP & language servers |
| `extension_host` — extension system | `git` — Git integration |
| `theme` — theming system | `collab` — collaborative editing |
| `settings` — settings UI | `lsp` — language server protocol |
| `util` — shared utilities | `search` — project-wide search |
| `fs` — filesystem abstraction | `go_to_line` and similar IDE actions |
| `menu` — menu bar | `vim` — Vim mode |
| `keymap` — keyboard shortcuts | `copilot` — GitHub Copilot |

**The guiding rule:** if it's about editing code, it goes. If it's about rendering, windowing, or user input, it stays.

---

## New Crates to Add

| Crate | Layer | Purpose | MVP? |
|-------|-------|---------|------|
| `matterhorn_composer` | L1 | Unified input bar | ✅ |
| `matterhorn_orchestrator` | L2 | Intent parsing + routing + planning | ✅ |
| `matterhorn_wallet` | L3 | Key management + signing | ✅ |
| `matterhorn_viewport` | L5 | WebView embedding + dApp injection | ✅ |
| `matterhorn_sidebar` | L5 | AI context panel | ✅ |
| `matterhorn_depin` | L4 | DePIN AI mesh + plugin protocol | ❌ Post-MVP |
| `matterhorn_common` | — | Shared types, errors, config | ✅ |

---

## Onboarding Flow

First-launch experience:

1. **Welcome screen** — "Browse Web3. Natively." + [Get Started]
2. **Wallet setup** — "Create new wallet" or "Import seed phrase"
3. **Password** — Encrypt seed with a local password
4. **Default networks** — Preconfigure Ethereum + Solana mainnets. Option to add L2s.
5. **Personality** — "What do you use Web3 for?" (DeFi / NFTs / DePIN / All) — tunes default dApp suggestions
6. **Done** — Drop into the composer with a placeholder: "Ask anything or enter a URL..."

---

## MVP Scope (Layers 1–3 + 5, minus DePIN)

What ships for May 12 launch:

- [x] L1: Unified composer with URL + basic NL input
- [x] L2: Intent parser with `navigate` + `search` intents (defer `transact` + `multi_step`)
- [x] L3: Wallet generation, Keychain storage, balance display (defer transaction signing)
- [x] L5: dApp viewport (WebView-based), tab management, AI sidebar (read-only)

What is post-MVP:

- [ ] Full transaction signing and confirmation (L3 complete)
- [ ] DePIN AI mesh (L4)
- [ ] Multi-step execution planner (L2 `multi_step`)
- [ ] Hardware wallet support
- [ ] WalletConnect v2
- [ ] Mobile companion app

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (matching Zed's codebase) |
| UI | GPUI (Zed's GPU-accelerated UI framework) |
| Web rendering | `wry` (WebView) or `webview` crate — Tauri's underlying engine |
| EVM wallet | `alloy-rs` (alloy providers, signers, transports) |
| Solana wallet | `solana-sdk`, `solana-client` |
| Key storage | `security-framework` (macOS), `windows-rs` (Windows) |
| HTTP | `reqwest` (already used in Zed) |
| LLM integration | OpenAI-compatible REST API, provider-agnostic |
| Async runtime | `tokio` (already Zed's runtime) |

---

## Key Design Decisions

1. **WebView, not custom renderer (for MVP).** Building a Web3-specific rendering engine is a multi-year project. Embed a WebView and focus differentiation on the composable surface + wallet integration + AI context.

2. **Native wallet, no extension bridge.** The wallet lives in the same process as the renderer. This is the core architectural bet. It means no `window.ethereum` polyfill over IPC — the wallet is genuinely in-process.

3. **LLM-provider-agnostic.** Don't hardcode OpenAI. The settings panel should accept any OpenAI-compatible endpoint. Users can bring their own API key or use a default provider.

4. **GPU-accelerated UI from day one.** GPUI is the moat. Smooth 120fps rendering of browser chrome, wallet confirmations, and AI sidebar — while a heavy dApp runs in the WebView below.

5. **DePIN is the differentiator, but not the MVP.** "Decentralized intelligence" is the long-term vision. For launch, a single configurable AI endpoint + curated on-chain data sources is enough.

---

## Build Order (for OpenCode)

### Phase 1: Skeleton (`matterhorn_browser` binary)

1. Fork from current `matterhornso/zed` main branch
2. Create new binary target `matterhorn_browser` in workspace `Cargo.toml`
3. Create `matterhorn_common` crate — shared types, error enum, config struct
4. Strip IDE crates from default workspace members
5. Verify: `cargo build` produces a window that opens (empty GPUI window)

### Phase 2: Viewport (L5 core)

6. Add `wry` dependency + create `matterhorn_viewport` crate
7. Implement basic WebView embedding in a GPUI view
8. Tab management: port Zed's workspace tab model to browser tabs
9. URL bar: simple text input that navigates the WebView on Enter
10. `Cmd+T` new tab, `Cmd+W` close tab, `Cmd+L` focus URL bar

### Phase 3: Composer (L1)

11. Create `matterhorn_composer` crate
12. Replace simple URL bar with unified composer input
13. Implement mode detection (URL vs NL heuristics)
14. History/suggestions dropdown
15. Command palette integration (`Cmd+K`)

### Phase 4: Wallet (L3)

16. Create `matterhorn_wallet` crate
17. BIP39 mnemonic generation + BIP44 derivation
18. Keychain/DPAPI storage via `security-framework`
19. Balance display (query RPC, show ETH/SOL balances)
20. Address display with ENS/SNS resolution
21. Onboarding flow UI (welcome → create/import → password → done)

### Phase 5: Orchestrator (L2)

22. Create `matterhorn_orchestrator` crate
23. LLM client module (configurable endpoint, JSON schema enforcement)
24. Intent parser: classify input → structured intent
25. Router: map intent → action (navigate/search only for MVP)
26. Connect composer → orchestrator → viewport pipeline

### Phase 6: Sidebar + Polish (L5 remaining)

27. Create `matterhorn_sidebar` crate
28. AI context panel (current tab info, recent actions)
29. Toggle with `Cmd+B`
30. Window chrome: back/forward/reload buttons, tab bar styling
31. Dark theme matching Matterhorn brand (`#0C0C0C` background, `#D1F2FF` accent)

### Phase 7: Wallet Complete (L3 remaining, post-MVP)

32. Transaction building + signing (EVM via alloy-rs)
33. Inline confirmation sheet (GPUI view)
34. Human-readable transaction rendering
35. Post-tx toast with block explorer link

---

## Brand & Visual Guidelines

- **Background:** `#0C0C0C` (near-black, like the matterhorn-site)
- **Accent:** `#D1F2FF` (icy blue)
- **Text:** `#FFFFFF` primary, `#A1A1A6` secondary
- **Border:** `#2C2C2E` subtle borders
- **Typography:** System font stack (SF Pro on macOS, Segoe UI on Windows)
- **Corner radius:** 8px for cards/sheets, 6px for buttons, full-round for input bar
- **Tone:** Confident, builder-first. No emoji in body copy. No exclamation marks.

---

## Success Criteria (MVP)

- [ ] App opens to composer surface
- [ ] Type a URL → navigates WebView to that site
- [ ] Type a natural-language query → sends to LLM, shows results
- [ ] Wallet: generate new wallet, see balance, see address with ENS
- [ ] Onboarding flow completes without terminal intervention
- [ ] Tabs work: new, close, switch
- [ ] Sidebar toggles
- [ ] Runs at 60fps+ on Apple Silicon

---

## File structure (target)

```
zed/
├── crates/
│   ├── gpui/                      # KEPT — GPU UI framework
│   ├── workspace/                 # KEPT — but stripped to browser-tab semantics
│   ├── theme/                     # KEPT
│   ├── settings/                  # KEPT
│   ├── util/                      # KEPT
│   ├── matterhorn_common/         # NEW — shared types, errors, config
│   ├── matterhorn_composer/       # NEW — L1 unified input
│   ├── matterhorn_orchestrator/   # NEW — L2 AI engine
│   ├── matterhorn_wallet/         # NEW — L3 wallet & identity
│   ├── matterhorn_viewport/       # NEW — L5 WebView + dApp injection
│   ├── matterhorn_sidebar/        # NEW — L5 AI context panel
│   └── matterhorn_depin/          # NEW — L4 DePIN mesh (post-MVP)
├── src/
│   └── main.rs                    # matterhorn_browser binary entrypoint
├── Cargo.toml                     # workspace manifest
└── README.md                      # updated with fork details
```

---

## References

- **Matterhorn 2.0 product:** [matterhorn.so](https://matterhorn.so) — "Cowork for Web3"
- **Matterhorn 2.0 site codebase:** `/Users/thebiglebowski/matterhorn/matterhorn-site/`
- **Zed source:** `https://github.com/zed-industries/zed` (the fork base)
- **GPUI docs:** within Zed's `crates/gpui/` — entity/context/view system
- **alloy-rs:** `https://github.com/alloy-rs/alloy` — Ethereum SDK
- **solana-sdk:** `https://docs.rs/solana-sdk` — Solana SDK
- **wry:** `https://github.com/nickkuk/wry` — WebView for Rust (Tauri's engine)
