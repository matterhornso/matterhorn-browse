# Matterhorn Browse

> **A Web3-native browser built on the Zed editor.** Fork of [zed-industries/zed](https://github.com/zed-industries/zed) by [Matterhorn](https://matterhorn.so).

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](./LICENSE-AGPL)
[![Fork of Zed](https://img.shields.io/badge/fork%20of-zed--industries%2Fzed-brightgreen)](https://github.com/zed-industries/zed)

---

## What is Matterhorn Browse?

Matterhorn Browse is an open-source, GPU-accelerated **Web3 browser** for macOS (with Linux and Windows planned). It combines:

- **A built-in self-custody wallet** — EVM and Solana. Keys live in your OS Keychain
- **A unified composer surface** — One input bar for URLs, natural-language questions, and transaction intents. The browser figures out what you meant
- **Reverse ENS resolution** — Your wallet shows its `.eth` name when one is registered
- **A GPU-accelerated rendering engine** — Based on GPUI, the same engine that powers Zed
- **A unified DePIN dashboard** — *(planned, post-MVP)* Manage Helium hotspots, Render jobs, Filecoin storage, and more from one place

Matterhorn Browse is for anyone curious about Web3 — from your first wallet to your tenth DePIN network.

---

## This is a Zed fork

Matterhorn Browse is built on **[Zed](https://github.com/zed-industries/zed)** — the high-performance, multiplayer code editor created by the team behind Atom and Tree-sitter. We're deeply grateful to [Zed Industries](https://zed.dev) for building such an incredible foundation and releasing it as open source under AGPL-3.0.

### What we kept from Zed

| Component | What it is | Why we kept it |
|-----------|-----------|----------------|
| **GPUI** | GPU-accelerated UI framework | Native 120fps rendering. ~50MB idle vs 400MB for Chromium. The technical moat |
| **Workspace + Tabs** | Multi-pane tab management | Browsing Web3 means many tabs — dapps, explorers, DePIN dashboards |
| **Extension system** | Plugin framework | Community-built DePIN integrations, chain explorers, and theme packs |
| **Terminal** | Embedded PTY | CLI-based DePIN tools (Helium CLI, Akash, Render CLI) run natively |
| **Collab** | Real-time collaboration primitives | Shared DePIN dashboards and multi-signature dapp sessions |
| **Theme system** | JSON-based syntax themes | Dark-mode optimized for blockchain data display |
| **Language support** | Tree-sitter syntax highlighting | `.sol` (Solidity), `.move` (Move), `.vy` (Vyper) syntax out of the box |

### What we changed

Matterhorn Browse is a browser, not an IDE. We stripped and replaced:

| Removed / Replaced | With |
|-------------------|------|
| Code editing (full) | Lightweight editor panel — config files + contract viewing only |
| Project panel | **Network explorer** — DePIN networks instead of file trees |
| Git integration | **Wallet activity** — transaction history instead of commit history |
| Agent AI panel (primary) | Optional. Not the core UX |
| LSP for non-Web3 languages | **LSP for Solidity, Vyper, Move** only |
| Debugger for C/Rust/etc | **On-chain debugger** — transaction tracing, event decoding |

### What we added

These are the new crates that make Matterhorn Browse what it is:

```
crates/
  matterhorn_browser/       ← Binary entrypoint (GPUI window)
  matterhorn_common/        ← Shared types, errors, persisted config
  matterhorn_composer/      ← L1 — Unified input bar (URL/NL/TX detection)
  matterhorn_orchestrator/  ← L2 — LLM intent parser + router
  matterhorn_wallet/        ← L3 — Self-custody wallet (EVM via k256, Solana via ed25519, BIP39, Keychain)
  matterhorn_onboarding/    ← Onboarding & unlock (create / import / unlock)
  matterhorn_viewport/      ← L5 — wry WebView, tabs, navigation, transaction confirmation sheet
  matterhorn_sidebar/       ← L5 — AI context panel (Cmd+B)
```

> A `matterhorn_depin` crate is planned post-MVP — see the spec at [docs/matterhorn-browser-spec.md](./docs/matterhorn-browser-spec.md).

---

## Why a Web3 browser?

Existing Web3 tools fall into silos:

- **Brave** is a privacy browser with a wallet tacked on. No DePIN. No native chain exploration
- **MetaMask / Phantom** are browser extensions. They don't render pages, manage tabs, or show you on-chain data outside of signing
- **Etherscan / Solscan** are read-only explorers in a browser tab. No wallet, no interaction layer
- **Every DePIN network** has its own dashboard at a different URL with a different login

Matterhorn Browse is the first application that combines all three — browser, wallet, and DePIN dashboard — into one native desktop experience.

---

## DePIN integrations (planned, post-MVP)

These targets are in the roadmap but **not yet implemented**. The plugin framework that drives them is the post-MVP `matterhorn_depin` crate (Layer 4 in the spec).

| Category | Networks | What you'll be able to do |
|----------|---------|-----------------|
| **Compute** | Render, Akash, io.net, Golem | Browse GPU/CPU availability, submit jobs, monitor earnings |
| **Storage** | Filecoin, Arweave, Storj | Upload files, browse storage deals, manage provider nodes |
| **Wireless** | Helium, Helium Mobile, DIMO | Manage hotspots, track coverage, monitor device earnings |
| **Mapping** | Hivemapper | Browse map coverage, manage dashcam contributions, track token rewards |
| **Energy** | Daylight, Powerledger | Monitor energy production, browse P2P energy marketplace |

All integrations will be built through a plugin framework — any DePIN network will be able to add a first-class dashboard.

---

## Installation

Pre-built binaries will be on the [releases page](https://github.com/matterhornso/matterhorn-browse/releases) once we cut the first release. Until then, build from source:

### macOS (primary target — wallet uses Keychain via `security-framework`)
```bash
./script/bootstrap             # install build dependencies
cargo build --release -p matterhorn_browser
open target/release/matterhorn_browser
```

### Linux / Windows
The wallet currently depends on macOS Keychain. Linux (Secret Service / KWallet) and Windows (DPAPI) backends are tracked as post-MVP work.

See [Zed's development docs](https://github.com/zed-industries/zed#developing-zed) for system-dependency setup that the bootstrap script doesn't cover.

### Configuration

Matterhorn writes a config file to `~/.matterhorn/config.json` on first launch. Edit it to point at your own RPC endpoints or LLM provider — the schema:

```json
{
  "llm_endpoint": "https://api.openai.com/v1",
  "llm_model": "gpt-4o",
  "llm_api_key": null,
  "ethereum_rpc": "https://eth.llamarpc.com",
  "solana_rpc": "https://api.mainnet-beta.solana.com"
}
```

---

## Related projects

- **[Matterhorn 2.0](https://matterhorn.so)** — "Cowork for Web3." A full agentic workspace where skills, tools, wallets, and chains compose. Launches May 2026
- **[Zed](https://zed.dev)** — The incredible GPU-accelerated code editor this browser is built on
- **[gstack](https://github.com/garrytan/gstack)** — Garry Tan's AI engineering workflow. Used in planning Matterhorn Browse

---

## Contributing

Matterhorn Browse is open source (AGPL-3.0, matching Zed's license). We welcome contributions:

1. **DePIN integrations** — Add your network via the plugin framework
2. **Chain support** — Extend wallet support to Cosmos, Bitcoin, Move-based chains
3. **Extensions** — Build dapp connectors, chain explorers, and theme packs

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

---

## License

AGPL-3.0, matching Zed's upstream license. See [LICENSE-AGPL](./LICENSE-AGPL).

---

## Acknowledgements

- **[Zed Industries](https://zed.dev)** — for building Zed, GPUI, and the collaboration infrastructure this browser is built on. Zed is an extraordinary piece of engineering
- **[Zed contributors](https://github.com/zed-industries/zed/graphs/contributors)** — the 1,700+ people who've contributed to Zed
- **[Garry Tan / gstack](https://github.com/garrytan/gstack)** — AI engineering workflow used to plan this project
