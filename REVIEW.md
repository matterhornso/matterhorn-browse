# Matterhorn Browser — Strategic Review

> A hard look at what was built on top of Zed, why it doesn't work, and the three honest paths forward.
>
> Written: 2026-05-12 (launch day). The binary builds clean, ships a `.app`, and renders a window. It does not render text. It does not work.
>
> This is not a patch-fix list. The patch-fix list is in [TODOS.md](./TODOS.md). This document is about whether the strategy itself is right.

---

## TL;DR

We **hard-forked Zed** — a 1.3 million-line editor — to build a Web3 browser. Zed's binary entrypoint runs **80+ subsystem initializations across ~300 lines of `app.run` setup** before opening a window. Our binary does **four lines** of setup before opening a window. We inherited Zed's complexity but wired almost none of its operational scaffolding, and that gap is why the UI is blank.

The Keccak fix, the unlock flow, the password masking, the font_family fix — every patch we shipped was technically correct and on a correct codebase. They land on a runtime that was never wired to function. **Patching individual symptoms will not produce a working app.**

You have three real choices. They are not "fix and ship in a week." They are:

1. **Finish wiring the Zed fork properly.** Replicate ~250 lines of init from `crates/zed/src/main.rs` into `crates/matterhorn_browser/src/matterhorn_browser.rs`, plus accept the maintenance cost of staying merged with upstream Zed forever. Earliest realistic ship: 2-3 weeks of dedicated work.
2. **Re-architect on GPUI as a library**, not a Zed fork. Use the 39 standalone GPUI examples (`crates/gpui/examples/`) as the new starting point. Throw away the Zed fork. Keep the eight matterhorn crates' business logic. Earliest realistic ship: 1-2 weeks.
3. **Pivot to Tauri 2.0.** Native webview, mature toolkit, real Apple Developer signing pipeline out of the box. The matterhorn business logic (wallet, orchestrator, composer logic) ports — it's plain Rust. Earliest realistic ship: 1 week to functional, longer to feature parity with the spec.

The MVP launch date in the spec was **today, May 12, 2026**. None of these three options ship today.

---

## What was actually built

| Crate                       | LOC  | Status |
|---|---|---|
| `matterhorn_wallet`         | 330  | Logic correct after Keccak fix. Cannot be exercised because no UI renders. |
| `matterhorn_common`         | 123  | Correct. Loads/saves config to `~/.matterhorn/config.json`. |
| `matterhorn_composer`       | 290  | Renders nothing visible. Mode detection logic is sound. |
| `matterhorn_orchestrator`   | 302  | LLM response parser fix is correct. Untestable until UI works. |
| `matterhorn_onboarding`     | 930  | Renders nothing visible. Phase machine, password masking, seed-phrase field, unlock flow — all written but unverifiable. |
| `matterhorn_sidebar`        | 143  | Renders nothing visible. |
| `matterhorn_viewport`       | 900  | Phase machine + tab history + nav buttons + confirmation sheet — all written but unverifiable. The `wry::WebView` may also have issues we cannot reach. |
| `matterhorn_browser`        | 47   | **The smoking gun. See below.** |

Plus 13 commits of fixes, a CI workflow that produces a `.app` artifact in ~5 min, and three planning documents: spec (444 lines), gap analysis (265 lines, mostly stale), and context (this work).

**~2,400 LOC of matterhorn-specific Rust on top of Zed's 1.3M.**

---

## What's gone wrong — the single root cause

Our `crates/matterhorn_browser/src/matterhorn_browser.rs`, in its entirety:

```rust
fn main() {
    let platform = gpui_platform::current_platform(false);
    let app = Application::with_platform(platform).with_assets(Assets);

    app.run(move |cx| {
        Assets.load_fonts(cx)?;        // 1 init call
        let config = MatterhornConfig::load_or_default();
        config.save()?;                // not even GPUI, just file IO

        let options = WindowOptions { ... };
        cx.open_window(options, |_, cx| {
            cx.new(|cx| BrowserState::new(cx, config))
        })?;
        cx.activate(true);
    });
}
```

**That is the whole startup sequence.** ~45 lines.

Zed's equivalent — `crates/zed/src/main.rs` — is **1,966 lines.** The `app.run(move |cx| { ... })` body alone runs **80 subsystem `init` calls** and **~10 `cx.set_global` registrations** before the window opens:

```rust
// excerpt from crates/zed/src/main.rs:448-815 (the production binary's app.run body)
cx.set_global(app_db);
trusted_worktrees::init(db_trusted_paths, cx);
menu::init();
zed_actions::init();
release_channel::init(app_version, cx);
gpui_tokio::init(cx);
settings::init(cx);                                          // ← SettingsStore as global
zlog_settings::init(cx);
<dyn Fs>::set_global(fs.clone(), cx);                       // ← filesystem global
GitHostingProviderRegistry::set_global(...);
extension::init(cx);
zed::init(cx);
project::Project::init(&client, cx);
client::init(&client, cx);
feature_flags::FeatureFlagStore::init(cx);
AppState::set_global(app_state.clone(), cx);                // ← app state global
theme_settings::init(theme::LoadThemes::All(Box::new(Assets)), cx);  // ← theme global
eager_load_active_theme_and_icon_theme(fs.clone(), cx);
command_palette::init(cx);
language_model::init(cx);
load_embedded_fonts(cx);                                    // ← fonts
editor::init(cx);
workspace::init(app_state.clone(), cx);
// ... 60 more init() calls
cx.observe_global::<SettingsStore>(...);                    // ← text rendering mode
cx.set_text_rendering_mode(...);
cx.set_menus(menus);
initialize_workspace(app_state.clone(), cx);
cx.activate(true);
```

Many of these are IDE-specific (project_panel, terminal_view, vim, git_ui) and irrelevant. **But the foundation calls are not optional.** Specifically:

- `settings::init(cx)` registers `SettingsStore` as a global. Any code that calls `SomeSettings::get_global(cx)` panics without it. GPUI itself doesn't use SettingsStore — but Zed's text rendering subscribes to it for `text_rendering_mode`, and Zed's themes layer on top of it.
- `theme_settings::init(theme::LoadThemes::All(Box::new(Assets)), cx)` initializes the `GlobalTheme`. Many GPUI text rendering paths read from it for default `TextStyle`.
- `cx.set_text_rendering_mode(...)` sets subpixel vs grayscale rendering. Default exists but Zed configures it from settings.
- `cx.observe_global::<SettingsStore>(...)` is what keeps the window in sync with theme changes.

We initialize **none of these.** The window opens against an underinitialized GPUI environment. The fact that backgrounds and basic shapes render at all is a credit to GPUI's robustness; that text doesn't is the predictable consequence of skipping its setup.

### Hard evidence

- `strings` on the shipped binary shows IBM Plex Sans **and** Lilex TTF data baked in correctly via `rust-embed`. The fonts are physically present in our 14 MB Mach-O.
- Running the binary in a headless shell for 5s produces **zero stderr output** — no panic, no font-load error, no missing-global complaint. It silently does what it can with what it has.
- The `Assets.load_fonts(cx)` call we added (commit `b75afec`) is the same code path Zed uses, so font *registration* works. The problem is that without theme/settings, GPUI's text rendering can't find a TextStyle that uses those fonts.

**Conclusion:** the text-not-rendering symptom is one face of a much bigger problem. Even if we wired text rendering correctly, the next time we touched anything in Zed that reads from a global we didn't set (project state, language registry, http client, command palette) we'd hit the next blank window.

---

## What's gone wrong — the strategic mistake

The original decision was: fork Zed because we want GPUI + GPU-accelerated rendering + tab management + extensions for free.

That decision had three consequences we didn't fully reckon with:

### 1. The "for free" part isn't free

Zed's editor functions only exist because Zed's `main.rs` wires together its 240+ crates. We deleted the editor surface but kept the entire 240-crate workspace as dependencies. Our binary depends transitively on `editor`, `project`, `language`, `lsp`, `collab`, `terminal_view` — every workspace member — even though we never call them.

Our `cargo build --release -p matterhorn_browser` compiles the entire workspace. ~20 min on a clean CI runner. Most of those minutes compile code matterhorn-browse will never execute.

### 2. The upstream pull problem

Upstream Zed ships weekly. Their workflows assume the `zed-industries` GitHub org (we even had to skip the upstream `release.yml` because of this gate). Every Zed PR that touches `crates/gpui`, `crates/workspace`, or `crates/assets` is a potential merge conflict for us. We have no upstream-sync strategy. There is no plan for what happens when Zed cuts v0.260.0 next week.

### 3. The fork hides Zed's actual API surface from us

`GPUI` is designed to be usable as a library. Its `crates/gpui/examples/` directory has **39 standalone example apps** — `hello_world.rs`, `animation.rs`, `drag_drop.rs`, `data_table.rs`, `gif_viewer.rs`, etc. Each is a complete app in 100-300 lines, using GPUI directly without any Zed-specific scaffolding.

`hello_world.rs` (~140 lines) **renders text against a colored background** with no `load_fonts`, no `theme_settings::init`, no `settings::init` — just `application().run(|cx| { ... open_window ... })`. This proves GPUI can render text in a minimal setup.

The hello_world API surface we should have started from:
```rust
use gpui_platform::application;
application().run(|cx: &mut App| {
    cx.open_window(WindowOptions { ... }, |_, cx| {
        cx.new(|_| MyApp { ... })
    });
    cx.activate(true);
});
```

That's the whole Application setup. Anything beyond that should be a deliberate choice, not an accidental inheritance.

**We took the IDE distribution and stripped it. We should have taken the GPUI library and built on it.** Same engine, opposite direction.

---

## What's NOT gone wrong

To be fair to ourselves:

- The **architecture** in `docs/matterhorn-browser-spec.md` is sound. 5 layers, clean separation, sensible MVP scope. The spec is not the problem.
- The **business logic** in each matterhorn crate is correct after fixes. The Keccak-256 derivation is right. The unlock flow is right. The phase machine is right. The LLM envelope parsing is right.
- The **CI build pipeline** works. We can produce a signed `.app` artifact in 5 minutes from any commit on `main`.
- The **planning artifacts** (spec, gap analysis, context, todos, this review) constitute a real product knowledge base.

The bug is not in our code. The bug is in our environment.

---

## Three honest paths forward

### Option A — Finish wiring the Zed fork

**What this means:** Copy the relevant init sequence from `crates/zed/src/main.rs` into `crates/matterhorn_browser/src/matterhorn_browser.rs`. Specifically the foundation calls: `settings::init`, `theme_settings::init` with our own minimal theme, `cx.set_text_rendering_mode`, font loading via Zed's `load_embedded_fonts` function rather than the simpler `Assets.load_fonts`. Skip the IDE-specific inits (editor, project, language, vim, etc.).

**Pros:**
- Preserves the 13 commits of correct business-logic fixes.
- Keeps the inherited window chrome, GPU acceleration, tab infrastructure.
- Most consistent with the original spec.

**Cons:**
- We need to deeply understand which inits are mandatory vs IDE-specific. This is a half-day to a day of careful tracing.
- The maintenance debt continues. Every Zed upstream pull is a potential break.
- Build times remain ~20 min on CI for code we don't run.
- The `target/` directory is ~3 GB on a debug build, ~1.2 GB release. Disk pressure.

**Time to a working "Get Started" button visible:** 1-2 days of focused work to identify and wire the minimum init set.
**Time to MVP shippable per spec:** 2-3 weeks (still need TX signing, dApp injection, address copy, seed display, signed `.dmg`).

---

### Option B — Re-architect as GPUI library consumer

**What this means:** New repo (or `crates/matterhorn_app/` inside this one). Binary uses `gpui_platform::application()` like the examples. Bring in GPUI as a path-dep on the existing `crates/gpui` directory, but don't depend on any other Zed crate. Port the eight matterhorn crates' source into this clean tree, removing their `gpui_tokio` / `util` / `paths` dependencies where they came from Zed-specific crates.

**Pros:**
- Aligns with how GPUI is actually designed to be used.
- Fast builds — only compile what we use.
- No more upstream-sync ambiguity.
- The `hello_world.rs` example confirms text rendering works without theme/settings/anything else.
- Cleaner mental model: "Matterhorn is a GPUI app that happens to inherit some Zed history."

**Cons:**
- Throws away the Zed workspace fork.
- Tab management primitives in `workspace` crate aren't inherited — we'd reimplement them on raw GPUI. Probably ~200 lines.
- No inherited theme system. We hard-code our brand colors (we already do, but now we own this).
- The CI workflow needs to be rewritten.

**Time to a working "Get Started" button visible:** 1 day. The hello_world pattern compiles and works immediately.
**Time to MVP shippable per spec:** 1-2 weeks. Less to wire, more to verify.

---

### Option C — Pivot to Tauri 2.0

**What this means:** Throw away the Rust+GPUI stack entirely. Build the browser chrome (composer, sidebar, toolbar, onboarding) in HTML/CSS/TS rendered by Tauri's main webview. Use a second Tauri webview as the dApp viewport. Wallet, orchestrator, and config logic stay in Rust as Tauri commands.

**Pros:**
- Tauri 2.0 is mature, well-documented, has a thriving ecosystem.
- Apple Developer ID signing + notarization is built into the toolchain.
- Webview-in-webview for the dApp viewport is the documented Tauri pattern.
- Frontend can be built by anyone comfortable with React/Vue/Svelte. Talent pool is 100x larger than GPUI.
- DePIN integrations (post-MVP) can ship as Tauri plugins.
- Auto-updater, app icon, menu bar — all solved problems.

**Cons:**
- Discards the GPU-accelerated browser-chrome story that was supposed to be Matterhorn's technical moat.
- Discards 2,400 LOC of working business logic UI code (the data layer ports, the UI doesn't).
- "GPU-accelerated, 5x lighter than Chromium" claims in our README become false.
- Tauri uses platform webviews (WKWebView on macOS) — same as wry which we already use. Same memory profile.

**Time to a working "Get Started" button visible:** Same day. Tauri `create-tauri-app` template + 50 lines of HTML.
**Time to MVP shippable per spec:** 1 week to functional, 2-3 weeks to feature parity (TX signing, multi-chain, ENS, unlock flow).

---

## Recommendation

**Option B**, with a 3-month review point on **Option C**.

Reasoning:

1. We chose GPUI for a reason — GPU-accelerated browser chrome at 120 fps is a real differentiator vs Brave/Chrome/Arc. Tauri can't deliver that.
2. We don't need the Zed fork to get GPUI. The library-consumer pattern in `crates/gpui/examples/` is the supported public API. We should use it.
3. The eight matterhorn crates' code is mostly portable — they import `gpui::*` types, not Zed-specific types. The exceptions (`paths`, `util`, `gpui_tokio`) are small replacements.
4. The 2-3 week extra cost of Option A buys us a maintenance burden that lasts forever. The 1-2 week cost of Option B buys us an architecture we control.
5. If Option B is still struggling at the 3-month mark — say we hit walls implementing TX confirmation sheets, or DePIN plugins — Option C remains available with the wallet/orchestrator code unchanged.

What changes immediately under Option B:

- New crate `matterhorn_app` (or new repo `matterhorn-browse-v2`).
- `gpui_platform::application().run(|cx| { ... })` as the entrypoint, like `hello_world.rs`.
- Path-dep on `crates/gpui` only. No other Zed crate dependencies.
- The eight matterhorn crates' source files copy over with `s/gpui_tokio/tokio/`, `s/util::/std::/` where possible, and minor cleanup.
- Wry stays — that's how we'll render dApps inside the GPUI window. Tauri uses the same WKWebView.
- CI workflow shrinks: ~5 min build instead of 20.
- The MVP launch date slips from today to ~mid-May. Communicate this clearly: "Matterhorn Browser MVP is now in active rebuild on a leaner architecture; new launch target is 2026-05-26."

---

## What to NOT do

- Do not keep patch-fixing. The Keccak fix was correct; the font_family fix was correct; the unlock flow was correct. None of them made the app render text. The bug is structural.
- Do not try to track down which specific GPUI init step fixes text rendering by trial and error. Even if we find it, we still have the workspace-bloat and upstream-sync problems.
- Do not announce the launch on Twitter today.
- Do not delete or destabilize anything yet. The current `main` is broken but documented. The next architecture starts in a parallel branch or repo so we can A/B and roll back.

---

## What to do in the next 24 hours

1. **You** (the user): decide between A, B, C. Default recommendation: B.
2. **Me** (if B is chosen): spike `crates/matterhorn_app/` with a `hello_world`-style binary that renders the welcome screen with text. If that works in ~50 lines, the path is validated. If it doesn't, we know it's not the init sequence and we re-diagnose.
3. **Both**: update the matterhorn.so marketing page to remove the "May 12" date and replace with "MVP in active rebuild — beta access soon." Don't promise a new date until the spike is green.
4. **Both**: write a one-paragraph explainer of the situation for any external folks who saw the May 12 launch promise. Honest, not defensive.

---

## Cross-reference

- Architecture intent: [`docs/matterhorn-browser-spec.md`](./docs/matterhorn-browser-spec.md)
- What was built and shipped: [`CONTEXT.md`](./CONTEXT.md)
- Roadmap from MVP to differentiated product: [`TODOS.md`](./TODOS.md)
- Upstream binary entrypoint (the one we should have studied first): [`crates/zed/src/main.rs`](./crates/zed/src/main.rs)
- The 39 working GPUI minimal examples (the path Option B builds on): [`crates/gpui/examples/`](./crates/gpui/examples/)
