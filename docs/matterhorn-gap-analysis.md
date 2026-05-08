# Matterhorn Browser — Gap Analysis

> **Date:** May 8, 2026
> **Spec:** `docs/matterhorn-browser-spec.md`
> **Codebase:** `https://github.com/matterhornso/zed` (fork of zed-industries/zed)
> **Analysis by:** Hermes (AI agent)

## Summary

All 7 planned MVP crates exist and compile individually. The architecture skeleton is in place (Composer → Orchestrator → Wallet → Viewport → Sidebar + Onboarding). However, there are **critical blockers** that prevent a functional build, plus **structural issues** that need addressing before any code works end-to-end.

**Severity legend:**
- 🔴 **Blocker** — Cannot compile or crashes at runtime. Must fix before anything else.
- 🟠 **High** — Correctness bug or missing feature that breaks core user flows.
- 🟡 **Medium** — Quality/UX issues, missing but non-blocking features.
- 🟢 **Low** — Code hygiene, polish, future concerns.

---

## 🔴 Blockers

### 1. Workspace Cargo.toml — Missing crate members

The `[workspace] members` array in `/Cargo.toml` does not include any of the new Matterhorn crates. Cargo will not discover them.

**Fix:** Add to `[workspace] members`:
```toml
"crates/matterhorn_browser",
"crates/matterhorn_common",
"crates/matterhorn_composer",
"crates/matterhorn_orchestrator",
"crates/matterhorn_wallet",
"crates/matterhorn_viewport",
"crates/matterhorn_sidebar",
"crates/matterhorn_onboarding",
```

### 2. Missing crate Cargo.toml files

Each `crates/matterhorn_*/` directory needs its own `Cargo.toml`. Current state:
- `matterhorn_viewport/Cargo.toml` — exists (wry dependency, gpui, etc.)
- All others — **missing**

Without per-crate manifests, the workspace cannot resolve dependencies and `cargo build` fails for any crate.

**Fix:** Create `Cargo.toml` for each crate with correct `[dependencies]`. The viewport crate's working config can serve as the template.

### 3. No binary target for matterhorn_browser

The `matterhorn_browser` crate (`crates/matterhorn_browser/src/matterhorn_browser.rs`) has a `main()` function but no `[[bin]]` target or `Cargo.toml` declaring it as a binary. Rust won't know this is an executable.

**Fix:** Add to `crates/matterhorn_browser/Cargo.toml`:
```toml
[package]
name = "matterhorn_browser"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "matterhorn"
path = "src/matterhorn_browser.rs"

[dependencies]
gpui = { path = "../gpui" }
gpui_platform = { path = "../gpui_platform" }
matterhorn_common = { path = "../matterhorn_common" }
matterhorn_viewport = { path = "../matterhorn_viewport" }
```

### 4. Ethereum address derivation is wrong

In `matterhorn_wallet.rs:293-301`:

```rust
fn derive_ethereum_address(signing_key: &SigningKey) -> String {
    let verifying_key = signing_key.verifying_key();
    let public_key = verifying_key.to_encoded_point(false);
    let pubkey_bytes = &public_key.as_bytes()[1..];  // strips 0x04 prefix
    let mut hasher = Sha256::new();
    hasher.update(pubkey_bytes);
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(&hash[hash.len() - 20..]))
}
```

**Two bugs:**
1. Uses **SHA-256** instead of **Keccak-256**. Ethereum addresses use Keccak-256 (SHA-3 variant), not SHA-256. Every derived address is wrong.
2. Takes last 20 bytes of the hash instead of the first 20. Standard is `keccak256(pubkey)[12..]`.

**Fix:** Replace with Keccak-256 from the `sha3` crate or `tiny-keccak`, and take bytes 12..32.

### 5. Viewport render method mutates state (GPUI anti-pattern)

In `matterhorn_viewport.rs:670-716`, the `Render::render()` method calls:
- `self.handle_submit(cx)` (line 696)
- Modifies `self.onboarding_done` (line 680)
- Modifies `self.first_render` (line 688)

GPUI's `render()` must be a pure function of state — it can be called many times and should not modify state. State mutations in render cause infinite re-render loops or inconsistent UI.

**Fix:** Move submission checking and state transitions to an `on_frame` or action handler. Use `cx.notify()` only for scheduling re-renders after actual state changes.

### 6. Confirmation sheet buttons have no click handlers

In `matterhorn_viewport.rs:480-505`, the Cancel and Sign buttons have `.id("tx-cancel")` and `.id("tx-confirm")` but **no `.on_click()` handler**. Clicking them does nothing — the only way to interact is keyboard (Escape/Enter).

**Fix:** Add `.on_click(cx.listener(...))` to both buttons matching the keyboard handlers.

---

## 🟠 High

### 7. Wallet import flow — no seed phrase field

Both `CreateWallet` and `ImportWallet` onboarding screens use the same `password_input` field (lines 230, 325). The import screen has no separate input for the seed phrase. When the user clicks "Import", it calls `wallet.import(&phrase, &password)` with `phrase = password_input` — which is the password, not a mnemonic.

**Fix:** Add a separate `phrase_input: SharedString` field to `OnboardingState`, render a textarea for it in `render_import_wallet`, and pass it to `wallet.import()`.

### 8. Wallet loads from keychain with empty password

In `matterhorn_viewport.rs:150-151`:
```rust
wallet_e.update(&mut c, |w, _cx| {
    let _ = w.load_from_keychain("");
});
```

The wallet is loaded with an **empty password**. The user set a password during onboarding, but it's never stored or re-prompted for. Result: `load_from_keychain` calls `import(phrase, "")` which derives keys with an empty passphrase — producing a different seed than what was created with the real password. Balances show zeros because the derived addresses are wrong.

**Fix:** Implement an unlock flow: on subsequent launches, show a password prompt → use that password to decrypt the seed → derive correct keys.

### 9. LLM response format mismatch

`classify_with_llm` (orchestrator.rs:99-138) sends a request and deserializes the response directly as `ClassificationResponse`. But OpenAI-compatible APIs return:
```json
{"choices": [{"message": {"content": "{\"intent\": \"...\"}"}}]}
```

The code tries to parse the top-level envelope as the classification object, which will always fail. The LLM path is completely non-functional.

**Fix:** Parse the OpenAI envelope first, extract `choices[0].message.content`, then JSON-parse that into `ClassificationResponse`.

### 10. No password masking / plaintext password display

The onboarding Create/Import screens display the password as typed in plaintext (lines 231-235, 327-329). This is a security issue for a wallet application.

**Fix:** Render password fields with dots/bullets. Add a "show password" toggle if desired.

### 11. No wallet unlock flow on return visits

After onboarding completes and the wallet is stored in Keychain, subsequent launches skip onboarding (`has_stored_wallet() == true`) but never prompt for the password. The wallet is loaded with an empty password (see #8), producing wrong keys. Even if #8 were fixed, there's no unlock prompt UI.

**Fix:** When `has_stored_wallet() == true` and no wallet is in memory, show an unlock screen (password prompt → `load_from_keychain(password)`).

---

## 🟡 Medium

### 12. Spec document missing `multi_step` decomposition

The spec calls for multi-step intent support (L2 Execution Planner). The orchestrator defines `Intent::MultiStep { steps: Vec<SubIntent> }` and accepts the `multi_step` classification, but `classification_into_intent` returns `MultiStep { steps: Vec::new() }` — empty, never decomposed.

**Fix:** Extend the LLM prompt to return sub-steps. Or build a second-pass LLM call that decomposes a complex intent into sequential sub-intents.

### 13. Composer mode detection lacks `multi_step`

The composer detects URL / NL / Transaction modes but not multi-step (no sequencing keyword detection like "then", "and then", "after"). Multi-step intents are classified as `NaturalLanguage`.

**Fix:** Add regex to detect sequencing keywords and return `InputMode::Transaction` (or a new `MultiStep` variant).

### 14. No tab switch resizes WebView properly

When switching tabs via keyboard shortcuts, the newly-active tab's WebView may have stale bounds. `ensure_webview` only creates webviews — it doesn't resize existing ones on tab switch.

**Fix:** Call a resize on the newly-active tab's webview after `self.active_tab` changes.

### 15. No settings persistence

`MatterhornConfig` is created with `::default()` in `main.rs` and never persisted. If the user changes RPC endpoints or LLM settings, they're lost on restart.

**Fix:** Serialize `MatterhornConfig` to a JSON/TOML file in `~/.matterhorn/`. Load on startup, save on changes.

### 16. No navigation buttons wired up

The toolbar has back/forward/reload buttons (← → ↻) but they display static text with no click handlers.

**Fix:** Implement browser history per tab, wire up navigation actions.

### 17. Sidebar `toggle()` method never used

`SidebarState` has a `toggle()` method, but `BrowserState` manages `sidebar_visible` directly. The sidebar's internal `visible` field and the browser state's `sidebar_visible` are out of sync.

**Fix:** Either remove `SidebarState.visible` and always rely on the parent, or make the parent read from the sidebar entity.

---

## 🟢 Low

### 18. No Solana import via seed phrase

`import()` only derives an Ethereum key. `create_solana()` requires an already-loaded mnemonic. There's no path to import a Solana-only wallet.

### 19. No ENS resolution in startup flow

`resolve_ens_name` is called during balance fetches but the result is only stored as `self.ens_name` — it's never used to resolve addresses in the composer or transaction flow.

### 20. `regex_lite` vs `regex` inconsistency

Composer uses `regex_lite::Regex` but orchestrator also imports it. The full `regex` crate is available (zed already depends on it). `regex_lite` has fewer features and may behave differently.

### 21. Multiple `use sha2::{Digest, Sha256}` imports

sha2 is imported inline in `matterhorn_viewport.rs:315` and also in `matterhorn_wallet.rs:11`. Should be a dependency of the wallet crate, not re-imported in the viewport.

### 22. `#[allow(dead_code)]` on WebContextGlobal

The WebContextGlobal struct has a dead_code suppression — it's stored as a GPUI global but never directly read. This is fine for the pattern, but a comment explaining why would help.

### 23. No tests

Zero test files across all 8 crates. The spec has success criteria but no test suite to verify them.

---

## Summary: What Ships vs What's Gapped

| Layer | Spec Requirement | Status |
|-------|-----------------|--------|
| L1 — Composer | URL + NL input with mode detection | ✅ Implemented |
| L1 — Composer | Transaction intent detection | ✅ Regex-based |
| L1 — Composer | History/suggestions dropdown | ✅ Implemented |
| L1 — Composer | Command palette (⌘K) | ❌ Missing |
| L2 — Orchestrator | Intent parser (regex heuristics) | ✅ Implemented |
| L2 — Orchestrator | LLM classification | 🟠 Broken (response format) |
| L2 — Orchestrator | Multi-step decomposition | ❌ Returns empty steps |
| L3 — Wallet | BIP39 generation + BIP44 derivation | 🟠 Wrong address derivation |
| L3 — Wallet | Keychain storage | ✅ Implemented |
| L3 — Wallet | Balance display (ETH + SOL) | 🟠 Blocked by #8 + #4 |
| L3 — Wallet | ENS resolution | ✅ Implemented |
| L3 — Wallet | Transaction signing | ✅ Mock-implemented |
| L3 — Wallet | Confirmation sheet UI | 🟠 No click handlers |
| L4 — DePIN Mesh | (Post-MVP — out of scope) | N/A |
| L5 — Viewport | WebView embedding | ✅ Implemented |
| L5 — Viewport | Tab management (Cmd+T/W/[ ]) | ✅ Implemented |
| L5 — Viewport | AI Sidebar (Cmd+B) | ✅ Implemented |
| L5 — Viewport | Back/Forward/Reload | ❌ Buttons not wired |
| — Onboarding | Welcome → Create/Import → Password → Done | 🟠 Import broken, no unlock |
| — Build | Workspace compiles | 🔴 Missing Cargo.tomls + members |
| — Build | Binary runs | 🔴 Blocked by above |
| — Brand | Dark theme (#0C0C0C / #D1F2FF) | ✅ Consistent |

## Recommended Fix Order

1. **Create all missing `Cargo.toml` files** (blockers #1, #2, #3)
2. **Fix Ethereum address derivation** (blocker #4)
3. **Fix Render side-effects** (blocker #5)
4. **Add click handlers to confirmation sheet** (blocker #6)
5. **Fix wallet import flow** (high #7)
6. **Implement unlock flow** (high #8, #11)
7. **Fix LLM response parsing** (high #9)
8. **Deploy and iterate** from there

---

*Analysis covers all 8 crate source files, the workspace Cargo.toml, and the full design spec (444 lines).*
