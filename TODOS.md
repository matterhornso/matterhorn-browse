# Matterhorn Browser — Roadmap

> The path from MVP to the actual product. Ordered by ship horizon, not by author opinion. Anything that gets us closer to "the browser the Web3 + AI audience actually wants to use" earns its place.
>
> Read [CONTEXT.md](./CONTEXT.md) first if you don't know what's already built.

**Audience we're building for:**
- A Web3 power user with 5+ wallets across 3+ chains who's tired of MetaMask popups
- A DePIN operator running 2-50 nodes who switches between Helium / Render / Filecoin dashboards daily
- A vibe-coder who wants their browser to be an agent, not a dumb URL bar
- An AI-native crypto-curious user who has never installed a wallet but is comfortable asking an LLM "help me get USDC on Base"

---

## P0 — Before MVP launch (2026-05-12)

Goal: someone other than us can install the .app, create a wallet, browse, and not lose money.

- [ ] **Verify a built `.app` actually launches.** As of this writing the binary builds in CI but a local install crashes on launch with no output. Either a dyld dependency mismatch (CI SDK newer than user macOS) or a Gatekeeper edge case. Repro and fix today.
- [ ] **Verify Ethereum addresses match a reference implementation** (`cast wallet`). Same seed + same password must produce the same address in Matterhorn and in MetaMask / foundry. The Keccak-256 fix lands in `029e77b` but it has not been roundtrip-verified against an external wallet.
- [ ] **Display the seed phrase to the user during create flow.** The wallet generates 12 words and discards them. A self-custody wallet that hides the seed is a fraud. Add a "Write these down" confirmation screen before storing in Keychain.
- [ ] **Address copy-to-clipboard.** Click the address in the toolbar → copy. Today it shows only ENS name or balances, not the address itself.
- [ ] **Apple Developer ID signing + notarization** for the `.app` so users don't need the right-click → Open dance. Requires Matterhorn's $99/yr Apple Developer cert. Until done, the install funnel is gated by Gatekeeper friction.
- [ ] **Releases page.** `gh release create v0.1.0` with the signed `.app`. Right now the README links to a releases page that doesn't exist.
- [ ] **`window.matterhorn` provider** stub. Dapps detect a wallet via `window.ethereum` (EVM) or `window.solana` (Solana). With no provider injection, every dApp shows "Install MetaMask" and our browser looks broken. Even a minimal provider that surfaces `accounts`, `chainId`, and `eth_requestAccounts` unlocks 80% of read-only dApp flows.
- [ ] **Crash → first-launch recovery.** If a panic leaves the GPUI window in a bad state, the user has no way to recover except `rm -rf ~/.matterhorn && security delete-generic-password -s com.matterhorn.browser.wallet`. Document this in README's troubleshooting section at minimum.

---

## P1 — First two weeks post-launch

Goal: people stay past the install.

### Wallet completeness
- [ ] **Real transaction signing for EVM.** Build RLP-encoded transactions via `alloy::rpc::types::TransactionRequest`, sign with the wallet's k256 key, broadcast via the configured RPC. The current "sign tx hash" path is mock.
- [ ] **Real transaction signing for Solana.** `solana-sdk` Transaction + ed25519 signing + RPC broadcast.
- [ ] **Transaction history per chain.** Pull recent txs from the RPC (`eth_getLogs` filtered to user's address, Solana `getSignaturesForAddress`). Show in the sidebar.
- [ ] **Gas estimation + fee preview** in the confirmation sheet. Today it says "Send 0.1 ETH to 0xabc..." with no fee, no USD value, no risk indicator. That's worse than MetaMask.
- [ ] **Multi-account support.** Most Web3 users have 3-10 addresses they switch between. Today we hard-code one Ethereum + one Solana account from m/44'/60'/0'/0/0 and m/44'/501'/0'/0'. Need an account-switcher UI and derive multiple accounts on demand.
- [ ] **Chain switching.** Today we hard-code Ethereum mainnet and Solana mainnet-beta. dApps on Base, Arbitrum, Optimism, Polygon, BNB Chain all expect the wallet to be on the right chain. Need a chain registry + UI for switching.
- [ ] **Hardware wallet support** (Ledger first). USB HID via `hidapi-rs`. Lots of high-value users will not put a hot key on a browser they just installed.
- [ ] **WalletConnect v2** for pairing with mobile dApps that don't have a browser injection path.
- [ ] **Export wallet / seed phrase recovery.** If a user wants to migrate to MetaMask or another wallet, they need their seed back. Today the seed is in Keychain but no UI surfaces it. Add a "Settings → Security → Reveal Recovery Phrase" flow with password reconfirmation.

### Browsing
- [ ] **Wire `Cmd+L` to actually focus an editable URL bar.** Today Cmd+L focuses the composer entity, but the composer renders the text as a static label (no real input element). Need a proper GPUI text input.
- [ ] **Address bar with URL editing in place** rather than always-blank composer. Most users expect to see and edit the current URL.
- [ ] **Bookmarks.** A first-class feature for Web3 users with 30+ dapps they cycle through.
- [ ] **Per-tab cookie isolation** for opsec — DeFi users absolutely will use the same browser for their main address and burner addresses, and they expect those identities not to bleed.
- [ ] **Reader mode for governance proposals.** Snapshot, Tally, Compound, Aave — proposals are walls of markdown. A reader-mode strip-to-text pass is high-value for governance-active users.
- [ ] **Find in page** (`Cmd+F`). Default browser feature. Currently missing.
- [ ] **Tab pinning** with a special render style for "always-open" dapps (Uniswap, Aave, the user's main DePIN dashboard).
- [ ] **Session restore.** Reopen tabs from last session on launch — power users have 15+ open tabs and don't want to rebuild every morning.

### Onboarding polish
- [ ] **Show recovery phrase confirmation.** After showing the 12 words, force the user to re-enter 3 of them in random positions before allowing "Start Browsing". Industry-standard friction; reduces the "I lost my seed" support burden.
- [ ] **Password strength meter.** Today any non-empty password is accepted, including `a`. That ships a wallet with effectively no encryption.
- [ ] **Biometric unlock** via Touch ID on macOS. After first password unlock, store an encryption key in the secure enclave keyed to Touch ID. Big UX win.

---

## P2 — First quarter post-launch

Goal: the differentiation features that the spec was actually built for.

### L2 — Orchestrator (the AI engine)
- [ ] **Multi-step execution planner.** The spec calls for a "plan card" UI: user types "bridge 500 USDC from Base to Arbitrum, then deposit into Aave", LLM decomposes into ordered sub-intents, user approves each step. Today the orchestrator returns `MultiStep { steps: Vec::new() }` — empty. Need both the LLM decomposition prompt and the UI to render the plan card.
- [ ] **Local LLM support.** Many Web3 users will not send "I want to buy 10 ETH" to OpenAI. Plug `ollama` (default endpoint `http://localhost:11434/v1`) as a config option. The orchestrator's `llm_endpoint` already accepts any OpenAI-compatible URL.
- [ ] **On-chain data tools** for the LLM. The classifier should be able to call functions like `get_balance(address)`, `get_price(token)`, `simulate_swap(in, out, amount)` rather than parroting whatever the user typed. Use tool-calling.
- [ ] **Transaction explanation.** When a dApp asks the wallet to sign a complex call, route the calldata through the LLM with the contract's ABI (fetched from Etherscan / Sourcify) and produce a human-readable summary in the confirmation sheet. This is the single biggest UX gap in EVM wallets.

### L3 — Identity beyond mnemonics
- [ ] **Account abstraction (ERC-4337) wallets** as an option alongside EOAs. Lower friction for new users (social recovery, gas sponsorship, batched txs).
- [ ] **Passkey-backed wallets.** Use platform passkeys (Touch ID / Windows Hello) as the signing key. No seed phrase to write down. Targets the "AI-native, crypto-curious" persona who has never used a wallet.
- [ ] **SIWE (Sign-In With Ethereum)** flow with one-click consent and a memory of which dApps the user has connected to. Today connecting is per-tab and forgotten.

### L4 — DePIN AI Mesh (post-MVP per spec, but ship at least one node)
- [ ] **`matterhorn_depin` crate.** L4 stub from the spec. Plugin protocol: JSON-RPC over WebSocket; nodes register with the browser; orchestrator queries them.
- [ ] **Helium integration.** Hotspot management, coverage map, daily earnings, mobile data routing. The Helium audience is one of the most concentrated DePIN user bases — shipping a working Helium dashboard alone is enough to drive installs.
- [ ] **Render Network integration.** GPU job submission, job status, earnings as a node operator.
- [ ] **Filecoin / Arweave storage browser.** Upload files, view your storage deals, monitor provider nodes.
- [ ] **A single "DePIN dashboard" home screen** for users who run multiple networks. Shows daily earnings rollup across all connected networks.

### L5 — Viewport and rendering
- [ ] **Tab groups for research sessions.** Tag tabs by chain or by intent ("Arbitrum DeFi research", "NFT mints to watch") and switch between groups.
- [ ] **Picture-in-picture transaction preview.** When a long swap is pending, show a small floating widget with the live confirmation count and gas paid.
- [ ] **Block explorer overlay.** Hover any `0x...` address on any page → popover with balance, token holdings, recent activity, ENS name. This is what Etherscan-as-a-browser-extension does, but native.

---

## P3 — Second half of 2026

Goal: the "moat" features that take real engineering.

### Performance
- [ ] **Sub-100ms tab open.** Most browsers spend 300-500ms creating a new tab. GPUI + wry should be able to hit 100ms with pre-warmed WebView contexts.
- [ ] **Lazy-decoded GPU textures** for tab thumbnails, so 50+ open tabs don't melt the GPU.
- [ ] **Native ad blocker** (uBlock-style filter lists). Web3 sites are increasingly ad-served; blocking is table-stakes.
- [ ] **Resource limits per tab** so a runaway dApp can't lock the whole browser.

### Web3-native rendering (the "wow" feature)
- [ ] **`.eth` / `.sol` URL bar resolution.** Type `vitalik.eth` → resolves to vitalik.eth's primary record → renders whatever's there (a redirect target, an IPFS site, a profile page). Today only the reverse direction works.
- [ ] **IPFS / Arweave / Lens content resolver.** `matterhorn_content` crate (planned, post-MVP). `ipfs://`, `ar://`, `lens://`, `farcaster://` schemes resolved natively.
- [ ] **EAS attestation rendering.** Any page can include an EAS attestation badge; we render it natively with the attestation source.
- [ ] **On-chain debugger.** Step through a transaction's execution trace inside the browser. Etherscan has this as a paid feature; we ship it free, integrated with the wallet.

### AI-native UX
- [ ] **Agent mode.** The browser as an autonomous agent. User says "find me the best APY on USDC across L2s and deposit my 5000 USDC", the agent visits each protocol, simulates, ranks, and surfaces a single confirmation sheet. The browser literally drives itself.
- [ ] **Long-running agent jobs** that the user can leave running and check on later (monitoring price targets, watching for governance proposals, sniping NFT mints).
- [ ] **AI-generated dApp summaries.** First time a user lands on a new dApp, the sidebar generates a one-paragraph "what is this and what does it want from you" summary, run locally if a local LLM is configured.

### Identity, social, communication
- [ ] **Native XMTP / Push Protocol inbox.** Web3 messaging is fragmented. Putting it in the browser sidebar makes the browser sticky.
- [ ] **Farcaster client.** Read + post + transact-from-cast directly in the browser sidebar.
- [ ] **Lens client.** Same idea.

### Multi-platform
- [ ] **Linux build.** Replace `security-framework` calls with a `secret-service` (Secret Service API) / `kwallet` backend, conditional-compile.
- [ ] **Windows build.** Replace with DPAPI via `windows-rs`.
- [ ] **iOS / Android companion app** for read-only viewing + push notifications on tx confirmations.

---

## P4 — Long game (2027+)

Speculative. Earns its way in only if the MVP gets traction.

- [ ] **Custom Web3 rendering engine** (not wry / WebKit). Renders dApps from on-chain manifests rather than fetching HTML over HTTP. Gives us the "no servers" Web3 story for real.
- [ ] **First-class cross-chain abstractions.** User asks for "1000 USDC on Base"; the browser figures out it's on Ethereum, bridges, swaps, settles, all through one confirmation sheet. The user never touches "chains" as a concept.
- [ ] **Token economics for community DePIN node operators** who run the L4 AI mesh.
- [ ] **Threshold-signed multisig wallets** via FROST / Schnorr. Send 10 ETH but only if your phone says yes after the browser says yes.
- [ ] **Plugin marketplace** for community DePIN integrations, chain explorers, dApp shortcuts. The extension system from Zed is already inherited; just need to plumb it.
- [ ] **Matterhorn 2.0 cowork integration.** Browser sessions sharable with collaborators in real-time using Zed's `collab` crate primitives.

---

## Maintenance and developer experience

These don't ship features, but they enable everything above.

- [ ] **Tests.** Zero exist today. Start with `matterhorn_wallet` unit tests for the Keccak fix and BIP44 derivation; add regression tests for every bug fixed in the May sprint.
- [ ] **`script/bundle-mac` adapted for matterhorn_browser** so we can produce a proper signed `.dmg` instead of just an `.app` zip.
- [ ] **Telemetry (opt-in).** We have no idea what users actually do. Crash reports + opt-in usage telemetry would 10x our ability to prioritize.
- [ ] **An issue template** that asks for: macOS version, wallet creation flow used, RPC endpoint, and console output from `~/Library/Logs/DiagnosticReports/`.
- [ ] **CI matrix** across macOS 12 / 13 / 14 / 15 so we catch SDK / dyld drift early.
- [ ] **A `cargo xtask` task** to regenerate the matterhorn-specific build/release workflows from a single source of truth, the way Zed does for its own workflows.
- [ ] **Documentation site.** `docs.matterhorn.so/browser` — at minimum: install, first wallet, importing from MetaMask, configuring an LLM endpoint. The marketing site is the entry point; docs are where users live.

---

## How to pick the next thing

When picking the next task, prefer in this order:
1. **Blocks the current user funnel** (someone today can't get past step N).
2. **Fixes a correctness issue users won't notice but that loses funds** (this list has too many post-launch entries that fit here; treat them like P0s when found).
3. **Differentiation from Brave / MetaMask / Phantom** — features none of them have and that we can ship in a sprint.
4. **Plumbing for a future feature in P2 or P3** — only if the next feature is already scheduled.

Avoid:
- Reformatting Zed's crates. We don't own them.
- "Improving" the LLM prompts in `matterhorn_orchestrator` without a benchmark suite to compare against.
- Pre-MVP polish on post-MVP features. The transaction confirmation sheet for example does not need MEV protection until it can sign real transactions.

---

## Cross-reference

- Authoritative design spec: [`docs/matterhorn-browser-spec.md`](./docs/matterhorn-browser-spec.md)
- Gap analysis (mostly closed): [`docs/matterhorn-gap-analysis.md`](./docs/matterhorn-gap-analysis.md)
- Current state of the code: [`CONTEXT.md`](./CONTEXT.md)
- Build workflow: [`.github/workflows/matterhorn_build_macos.yml`](./.github/workflows/matterhorn_build_macos.yml)
