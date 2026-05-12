# Matterhorn Browser — Current Context

> A working snapshot of what this codebase is, what's actually built, what was just shipped, and what's known-broken. Read this first when starting a session.
>
> Last updated: 2026-05-12.
>
> **Architecture pivoted from Zed-hard-fork to GPUI-as-library on 2026-05-12.** See [REVIEW.md](./REVIEW.md) for the full strategic review. The production binary is now **`matterhorn_app`** (in `crates/matterhorn_app/`), built via `gpui_platform::application().run(...)` — the same pattern as `crates/gpui/examples/hello_world.rs`. The legacy `matterhorn_browser` crate is deprecated but kept in-tree for diff and reference; do not build it.

---

## What this is

A **Web3-native desktop browser** forked from [zed-industries/zed](https://github.com/zed-industries/zed), built by [Matterhorn](https://matterhorn.so). Keeps Zed's GPUI rendering foundation, strips the IDE, adds a wallet, an LLM-driven composer, and a WebView-based viewport.

**Sister project:** Matterhorn 2.0 ("Cowork for Web3") — the same team's agentic workspace, launching alongside this browser.

**License:** AGPL-3.0 (matches upstream Zed).

---

## Architecture: 5 layers

Defined in `docs/matterhorn-browser-spec.md` (the source of truth for design).

```
┌──────────────────────────────────────────────────────────┐
│ L5  Execution & Rendering                                │
│     dApp viewport (wry WebView) · AI sidebar · TX sheets │
├──────────────────────────────────────────────────────────┤
│ L4  DePIN AI Mesh                          [post-MVP]    │
│     Search summarizers · Security · Tx simulation        │
├──────────────────────────────────────────────────────────┤
│ L3  Native Wallet & Identity                             │
│     BIP39 · k256 (EVM) · ed25519 (Solana) · Keychain     │
├──────────────────────────────────────────────────────────┤
│ L2  AI Orchestration Engine                              │
│     Intent parser · LLM classifier · Router              │
├──────────────────────────────────────────────────────────┤
│ L1  Unified Composer Surface                             │
│     One input bar · URL / NL / TX / multi-step           │
└──────────────────────────────────────────────────────────┘
```

---

## Crates we own

All matterhorn-specific code lives under `crates/matterhorn_*`. Total: **~2,400 LOC** of Rust on top of Zed's ~1.3M.

| Crate | Layer | Lines | Role |
|---|---|---|---|
| `matterhorn_app`         | bin  | ~30  | **Active binary entrypoint (Option B).** Uses `gpui_platform::application().run`, enters a tokio multi-thread runtime, mounts `BrowserState`. No Zed scaffolding. |
| `matterhorn_browser`     | bin  | 47   | **Deprecated.** The original hard-fork-of-Zed entrypoint that produced the blank UI. Kept in-tree until matterhorn_app is verified end-to-end, then will be removed. |
| `matterhorn_common`      | —    | 123  | `MatterhornError`, `MatterhornConfig`, config load/save at `~/.matterhorn/config.json`. |
| `matterhorn_composer`    | L1   | 290  | Unified input bar. URL/NL/Transaction mode detection. Cmd+K palette. History suggestions. |
| `matterhorn_orchestrator`| L2   | 302  | Intent parser (regex heuristics + LLM fallback). OpenAI-compatible chat completions client. |
| `matterhorn_wallet`      | L3   | 330  | BIP39 mnemonic, k256 ECDSA (Ethereum), ed25519 (Solana), macOS Keychain storage, ENS reverse-resolve via ensideas. |
| `matterhorn_onboarding`  | L3/UI| 930  | Welcome → Create / Import / Unlock flows. Seed-phrase input, password masking, Tab cycling. |
| `matterhorn_sidebar`     | L5   | 143  | AI context panel. Recent actions list. Cmd+B toggles. |
| `matterhorn_viewport`    | L5   | 900  | wry WebView, tab management, per-tab navigation history, toolbar, transaction confirmation sheet, phase machine (Onboarding → Unlocking → Browsing). |

**No `matterhorn_depin` crate exists yet.** DePIN is post-MVP per spec.

The names `mb_wallet`, `mb_depin`, etc. that appeared in early README drafts were never built. The README is now reconciled with reality.

---

## What's been built (as of 2026-05-11)

The repo has **10 commits** total. The original MVP commit (`adb6f7e`) landed the skeleton; commits since then closed every shipping blocker from `docs/matterhorn-gap-analysis.md`.

### Original MVP commit `adb6f7e` shipped
- Full 5-layer skeleton: composer → orchestrator → wallet → viewport, with onboarding and sidebar
- BIP39 wallet generation + Keychain storage
- ENS reverse-resolution (via ensideas)
- WebView embedding, tab management, AI sidebar
- Brand colors (`#0C0C0C` / `#D1F2FF`)
- 444-line spec + 265-line gap analysis docs

### Fix sprint commits (2026-05-09 → 2026-05-11)
| SHA | Subject | What it fixed |
|---|---|---|
| `029e77b` | fix(wallet): derive Ethereum addresses with Keccak-256 | Every address was wrong (SHA-256 + last 20 bytes). Now correct Keccak-256, bytes `[12..32]`. |
| `678d39c` | fix(orchestrator): parse OpenAI envelope before extracting classification | LLM path was dead. Now extracts `choices[0].message.content`, strips fences, parses JSON. |
| `714d663` | feat(common): persist `MatterhornConfig` at `~/.matterhorn/config.json` | LLM endpoint, model, API key, RPCs survive across launches. |
| `0cbf352` | feat(composer): wire Cmd+K command palette and live suggestions | Suggestions dropdown now opens on type, Escape closes, Cmd+K toggles. |
| `99f97b7` | feat(onboarding): seed-phrase field, password masking, unlock flow | Import had no phrase field; passwords were plaintext; relaunch had no unlock UI. All fixed. |
| `437a8c2` | refactor(viewport): phase machine, observers, navigation, sheet handlers | Replaced render-time mutations with `BrowserPhase` + `cx.observe`. Wired toolbar back/forward/reload with per-tab history. Tab click-to-switch + resize. Confirmation sheet click handlers. |
| `60a4708` | feat(browser): hydrate config from disk on launch | Reads `~/.matterhorn/config.json` instead of `Default::default()`. |
| `b931546` | docs: reconcile README with actual crates and shipped scope | Removed `mb_*` references; marked DePIN as planned. |
| `7c7664f` | chore(viewport): drop two pre-existing unused-warning sites | `let route` and `ORANGE` const. |
| `3f2b1ea` | ci: build matterhorn_browser on macOS in GitHub Actions | `workflow_dispatch` + on-push CI; produces a `.app` bundle artifact. |

The CI workflow at `.github/workflows/matterhorn_build_macos.yml` builds and uploads the `.app` so contributors without local Xcode can still produce a runnable binary.

---

## What still doesn't work (or is partial)

These are the carry-over items from the gap analysis that weren't critical for first-launch.

### Known partial / shipped-with-caveats
- **Wrong-password UX:** unlock screen shows "Unlock failed: ..." for any error, including wrong password. BIP39 derives a different (wrong) seed silently when the password is wrong, so a wrong password doesn't error — it produces a wallet with the wrong addresses. Mitigated only by verifying balance/address roundtrip on unlock.
- **Multi-step intent decomposition:** the orchestrator accepts `multi_step` from the LLM classifier but returns `MultiStep { steps: Vec::new() }`. Post-MVP per spec.
- **Transaction signing:** mock implementation only. Signs a SHA-256 of the tx fields, not a proper RLP-encoded Ethereum tx. Post-MVP per spec.
- **No Solana imports via seed phrase:** `wallet.import()` only derives the Ethereum key. To get a Solana account you need to call `create_solana()` after import, which the import button does. But there's no separate Solana-only import path.
- **No tests:** zero test files across all eight matterhorn crates. Spec doesn't require them for MVP.
- **No Windows / Linux Keychain backend:** `security-framework` is macOS-only. Wallet code won't compile on Linux/Windows today.

### Known gaps not in the original gap analysis
- **Seed phrase is never displayed to the user during create flow.** `wallet.create()` returns the phrase, but the UI throws it away. Users have no way to back up their seed. **This is a real risk for a self-custody wallet.**
- **No address copy-to-clipboard.** The toolbar shows ETH/SOL balances and the ENS name, but not the address itself in a copyable form.
- **No "show recovery phrase" / "export wallet" anywhere.** Users who lose their password are locked out with no recovery.
- **No real `window.matterhorn` provider injection.** The spec calls for a dApp-facing JS object similar to `window.ethereum`. Not built — dApps cannot detect or call the wallet.

---

## How to run / build

### Local build (requires full Xcode)
The GPUI renderer needs the Metal shader compiler, which only ships with **full Xcode** (not Command Line Tools alone). Then:

```bash
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license accept
xcrun -sdk macosx metal --version  # must print a version

cd matterhorn-browse
cargo build --release -p matterhorn_app
open target/release/matterhorn_app
```

`matterhorn_app` is also the workspace default member, so `cargo build --release` alone produces the right binary. Expected build time: ~5–10 min cold (much faster than the legacy `matterhorn_browser` which dragged in the full Zed workspace).

### Cloud build (no local Xcode)
Push to `main` or trigger the workflow manually:

```bash
gh workflow run matterhorn_app_macos.yml --ref main
gh run watch
```

The latest run's artifact contains `Matterhorn App.app` (ad-hoc signed). Download via:

```bash
gh run download <RUN_ID> -R matterhornso/matterhorn-browse
unzip matterhorn-app-macos-*/matterhorn-app-macos.zip
xattr -dr com.apple.quarantine "Matterhorn App.app"
open "Matterhorn App.app"
```

### Configuration
Config file: `~/.matterhorn/config.json` (auto-written on first launch). Schema:

```json
{
  "llm_endpoint": "https://api.openai.com/v1",
  "llm_model": "gpt-4o",
  "llm_api_key": null,
  "ethereum_rpc": "https://eth.llamarpc.com",
  "solana_rpc": "https://api.mainnet-beta.solana.com"
}
```

LLM intent classification (Cmd+K → natural-language input) requires `llm_api_key` to be set.

---

## Where things live

| Topic | Path |
|---|---|
| Design spec (authoritative) | `docs/matterhorn-browser-spec.md` |
| Gap analysis (mostly closed) | `docs/matterhorn-gap-analysis.md` |
| Implementation plan (legacy) | `matterhorn-browse-implementation-plan.md` |
| Pre-build research | `matterhorn-browse-research.md` |
| Build workflow | `.github/workflows/matterhorn_build_macos.yml` |
| Bundle script (Zed's, unused) | `script/bundle-mac` |
| Bootstrap script | `script/bootstrap` |
| Config file (runtime) | `~/.matterhorn/config.json` |
| Wallet storage (runtime) | macOS Keychain, service `com.matterhorn.browser.wallet` |

---

## Keyboard shortcuts (browsing phase)

| Shortcut | Action |
|---|---|
| `Cmd+T` | New tab |
| `Cmd+W` | Close active tab |
| `Cmd+L` | Focus composer |
| `Cmd+K` | Open command palette (suggestions dropdown) |
| `Cmd+B` | Toggle AI sidebar |
| `Cmd+[` / `Cmd+]` | Prev / next tab |
| `Cmd+R` | Reload active tab |
| `Esc` (in tx sheet) | Cancel pending transaction |
| `Enter` (in tx sheet) | Confirm transaction |

---

## Mental model for changes

- **The spec is authoritative.** When something in the README and the spec disagree, the spec wins.
- **The gap analysis is mostly stale.** Items #1–#3 (Cargo workspace blockers) were already fixed when the doc was written. Items #4–#11 (correctness/security blockers) were fixed in the May fix sprint. Verify any item against current code before acting on it.
- **Stay inside `crates/matterhorn_*`.** Do not refactor Zed's crates. The fork inherits Zed for free; rewriting any of it costs us all that.
- **`Render::render()` must be pure.** State transitions go through `cx.observe` callbacks, event handlers, or `cx.notify` — never inside `render`. The viewport refactor enforced this; keep it that way.
- **Keychain is the source of truth for the mnemonic.** Never write the mnemonic to disk. Never log it.
- **DePIN is post-MVP.** Don't add a `matterhorn_depin` crate yet. When you do, follow the L4 spec in `docs/matterhorn-browser-spec.md`.

---

## Useful gh commands

```bash
# Watch the most recent build
gh run list --workflow=matterhorn_build_macos.yml --limit 3
gh run watch

# Trigger a fresh build
gh workflow run matterhorn_build_macos.yml --ref main

# Download the latest .app
LATEST=$(gh run list --workflow=matterhorn_build_macos.yml --limit 1 --json databaseId --jq '.[0].databaseId')
gh run download $LATEST -R matterhornso/matterhorn-browse
```
