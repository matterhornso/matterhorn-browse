# Matterhorn Browse

> **A Web3-native browser built on the Zed editor.** Fork of [zed-industries/zed](https://github.com/zed-industries/zed) by [Matterhorn](https://matterhorn.so).

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](./LICENSE-AGPL)
[![Fork of Zed](https://img.shields.io/badge/fork%20of-zed--industries%2Fzed-brightgreen)](https://github.com/zed-industries/zed)

---

## What is Matterhorn Browse?

Matterhorn Browse is an open-source, GPU-accelerated **Web3 browser** for macOS, Linux, and Windows. It combines:

- **A built-in self-custody wallet** — EVM and Solana. Keys stay in your OS secure enclave
- **A unified DePIN dashboard** — Manage your Helium hotspots, Render Network jobs, Filecoin storage, and more from one place
- **Native on-chain browsing** — ENS, IPFS, and Arweave domains resolve natively in the URL bar
- **A GPU-accelerated rendering engine** — Based on GPUI, the same engine that powers Zed. 5x lighter than Chromium, instant startup

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
  mb_wallet/          ← Self-custody wallet (EVM + Solana)
  mb_depin/           ← Unified DePIN integration layer
  mb_tab/             ← Chain-aware tabs (dapps, explorers, dashboards)
  mb_onboarding/      ← Web3 onboarding (wallet creation, key backup, first token)
  mb_ens/             ← ENS / decentralized DNS resolution
  mb_content/         ← Content resolver (.ipfs, .arweave, .lens, .farcaster)
```

---

## Why a Web3 browser?

Existing Web3 tools fall into silos:

- **Brave** is a privacy browser with a wallet tacked on. No DePIN. No native chain exploration
- **MetaMask / Phantom** are browser extensions. They don't render pages, manage tabs, or show you on-chain data outside of signing
- **Etherscan / Solscan** are read-only explorers in a browser tab. No wallet, no interaction layer
- **Every DePIN network** has its own dashboard at a different URL with a different login

Matterhorn Browse is the first application that combines all three — browser, wallet, and DePIN dashboard — into one native desktop experience.

---

## DePIN integrations (planned)

| Category | Networks | What you can do |
|----------|---------|-----------------|
| **Compute** | Render, Akash, io.net, Golem | Browse GPU/CPU availability, submit jobs, monitor earnings |
| **Storage** | Filecoin, Arweave, Storj | Upload files, browse storage deals, manage storage provider nodes |
| **Wireless** | Helium, Helium Mobile, DIMO | Manage hotspots, track coverage, monitor device earnings |
| **Mapping** | Hivemapper | Browse map coverage, manage dashcam contributions, track token rewards |
| **Energy** | Daylight, Powerledger | Monitor energy production, browse P2P energy marketplace |

All integrations are built through a plugin framework — any DePIN network can add a first-class dashboard.

---

## Installation

On macOS, Linux, and Windows you can download Matterhorn Browse from the [releases page](https://github.com/matterhornso/matterhorn-browse/releases).

Or build from source:

### macOS
```bash
./script/bootstrap             # install build dependencies
cargo build --release          # ~5 min on M3, ~15 min on Intel
open target/release/Matterhorn\ Browse.app
```

### Linux
```bash
./script/bootstrap
cargo build --release
./target/release/matterhorn-browse
```

### Windows
```powershell
.\script\bootstrap.ps1
cargo build --release
.\target\release\matterhorn-browse.exe
```

See [Zed's development docs](https://github.com/zed-industries/zed#developing-zed) for detailed per-platform instructions.

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
