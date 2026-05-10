use std::rc::Rc;

use gpui::{
    div, prelude::*, px, rgb, Bounds, Context, Entity, Focusable, Global,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Pixels,
    Render, SharedString, Styled, Window,
};

use matterhorn_common::MatterhornConfig;
use matterhorn_composer::{ComposerState, InputMode};
use matterhorn_onboarding::OnboardingState;
use matterhorn_orchestrator::{Intent, MatterhornOrchestrator};
use matterhorn_sidebar::SidebarState;
use matterhorn_wallet::{MatterhornWallet, TransactionRequest};

const BG: u32 = 0x0C0C0C;
const SURFACE: u32 = 0x1C1C1E;
const SURFACE_ALT: u32 = 0x161618;
const BORDER: u32 = 0x2C2C2E;
const ACCENT: u32 = 0xD1F2FF;
const TEXT_MUTED: u32 = 0xA1A1A6;
const TEXT_DIM: u32 = 0x636366;
const TEXT: u32 = 0xFFFFFF;
const GREEN: u32 = 0x30D158;
const RED: u32 = 0xFF453A;

#[allow(dead_code)]
struct WebContextGlobal(wry::WebContext);

impl Global for WebContextGlobal {}

struct Tab {
    title: SharedString,
    url: SharedString,
    webview: Option<wry::WebView>,
    last_bounds: Bounds<Pixels>,
    /// Per-tab navigation history. `history[history_index]` is the current URL.
    history: Vec<SharedString>,
    history_index: usize,
}

impl Tab {
    fn new(title: impl Into<SharedString>, url: impl Into<SharedString>) -> Self {
        let url: SharedString = url.into();
        Self {
            title: title.into(),
            url: url.clone(),
            webview: None,
            last_bounds: Bounds::default(),
            history: vec![url],
            history_index: 0,
        }
    }

    fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    /// Push a new URL onto the history, truncating any forward stack.
    fn push_history(&mut self, url: SharedString) {
        if self.history.get(self.history_index) == Some(&url) {
            return;
        }
        self.history.truncate(self.history_index + 1);
        self.history.push(url);
        self.history_index = self.history.len() - 1;
    }
}

enum TxStage {
    Confirm(TransactionRequest),
    Signing,
    Signed { tx_hash: SharedString },
    Failed(SharedString),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum BrowserPhase {
    /// First-launch: no wallet stored, run create/import flow.
    Onboarding,
    /// Wallet stored in Keychain — prompt for password before browsing.
    Unlocking,
    /// Wallet ready, render browser UI.
    Browsing,
}

pub struct BrowserState {
    orchestrator: Rc<MatterhornOrchestrator>,
    composer: Entity<ComposerState>,
    onboarding: Entity<OnboardingState>,
    sidebar: Entity<SidebarState>,
    wallet: Entity<MatterhornWallet>,
    eth_rpc: String,
    sol_rpc: String,

    tabs: Vec<Tab>,
    active_tab: usize,

    eth_balance: SharedString,
    sol_balance: SharedString,
    ens_name: SharedString,

    sidebar_visible: bool,
    status_text: SharedString,
    phase: BrowserPhase,

    tx_stage: Option<TxStage>,
}

impl BrowserState {
    pub fn new(cx: &mut Context<Self>, config: MatterhornConfig) -> Self {
        let orchestrator = Rc::new(MatterhornOrchestrator::new(config.clone()));
        let composer = cx.new(|cx| ComposerState::new(cx));
        let wallet = cx.new(|_cx| MatterhornWallet::new());
        let has_stored = MatterhornWallet::has_stored_wallet();
        let onboarding = if has_stored {
            cx.new(|cx| OnboardingState::unlocking(cx, wallet.clone()))
        } else {
            cx.new(|cx| OnboardingState::new(cx, wallet.clone()))
        };
        let sidebar = cx.new(|_cx| SidebarState::new());
        let tabs = vec![Tab::new("Matterhorn", "https://matterhorn.so")];

        let phase = if has_stored {
            BrowserPhase::Unlocking
        } else {
            BrowserPhase::Onboarding
        };

        // Observe the onboarding entity. When it flips `done = true` (via the
        // create-wallet, import-wallet, or unlock flow), promote the phase to
        // Browsing and kick off balance fetches + WebView context.
        cx.observe(&onboarding, |this: &mut Self, onboarding, cx| {
            if onboarding.read(cx).done && this.phase != BrowserPhase::Browsing {
                this.transition_to_browsing(cx);
            }
        })
        .detach();

        // Observe the composer for new submissions.
        cx.observe(&composer, |this: &mut Self, composer, cx| {
            if composer.read(cx).submitted {
                this.handle_submit(cx);
            }
        })
        .detach();

        Self {
            orchestrator,
            composer,
            onboarding,
            sidebar,
            wallet,
            eth_rpc: config.ethereum_rpc,
            sol_rpc: config.solana_rpc,
            tabs,
            active_tab: 0,
            eth_balance: SharedString::from("..."),
            sol_balance: SharedString::from("..."),
            ens_name: SharedString::from("..."),
            sidebar_visible: false,
            status_text: SharedString::from(""),
            phase,
            tx_stage: None,
        }
    }

    fn transition_to_browsing(&mut self, cx: &mut Context<Self>) {
        self.phase = BrowserPhase::Browsing;
        let web_context = wry::WebContext::new(Some(std::env::temp_dir()));
        cx.set_global(WebContextGlobal(web_context));
        self.start_balance_fetches(cx);
        cx.notify();
    }

    fn ensure_webview(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        let active = self.active_tab;
        let tab = &mut self.tabs[active];
        if tab.webview.is_some() {
            return;
        }
        let url = tab.url.to_string();
        match wry::WebViewBuilder::new()
            .with_url(&url)
            .with_bounds(wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
                size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(800.0, 600.0)),
            })
            .build_as_child(window)
        {
            Ok(wv) => {
                self.tabs[active].webview = Some(wv);
            }
            Err(e) => {
                eprintln!("Failed to create WebView: {e}");
            }
        }
    }

    fn start_balance_fetches(&self, cx: &mut Context<Self>) {
        // The wallet is already loaded by this point — onboarding (create/import)
        // populated it from the entered password, or unlock decrypted the
        // mnemonic from Keychain. Fetch balances and reverse-resolve ENS only.
        let wallet_entity = self.wallet.downgrade();
        let eth_rpc = self.eth_rpc.clone();
        let sol_rpc = self.sol_rpc.clone();
        let mut c = cx.to_async();

        cx.spawn(|this: gpui::WeakEntity<Self>, _cx: &mut gpui::AsyncApp| async move {
            if let Some(wallet_e) = wallet_entity.upgrade() {
                let rpc = eth_rpc.clone();
                let balance = wallet_e.read_with(&c, |w, _| {
                    let handle = tokio::runtime::Handle::current();
                    handle.block_on(w.fetch_balance(&rpc))
                });
                if let Ok(bal) = balance {
                    this.update(&mut c, |s, cx| {
                        s.eth_balance = SharedString::from(bal);
                        cx.notify();
                    })
                    .ok();
                }
            }
            if let Some(wallet_e) = wallet_entity.upgrade() {
                let rpc = sol_rpc.clone();
                let balance = wallet_e.read_with(&c, |w, _| {
                    let handle = tokio::runtime::Handle::current();
                    handle.block_on(w.fetch_solana_balance(&rpc))
                });
                if let Ok(bal) = balance {
                    this.update(&mut c, |s, cx| {
                        s.sol_balance = SharedString::from(bal);
                        cx.notify();
                    })
                    .ok();
                }
            }
            if let Some(wallet_e) = wallet_entity.upgrade() {
                let addr =
                    wallet_e.read_with(&c, |w, _| w.selected_address().map(|a| a.to_string()));
                if let Some(address) = addr {
                    let name = wallet_e.read_with(&c, |w, _| {
                        let handle = tokio::runtime::Handle::current();
                        handle.block_on(w.resolve_ens_name(&address))
                    });
                    if let Some(n) = name {
                        this.update(&mut c, |s, cx| {
                            s.ens_name = SharedString::from(n);
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }
            anyhow::Ok(())
        })
        .detach();
    }

    fn handle_submit(&mut self, cx: &mut Context<Self>) {
        let submission = self.composer.update(cx, |comp, _cx| comp.take_submission());
        let (text, mode) = match submission {
            Some(s) => s,
            None => return,
        };
        if text.is_empty() {
            return;
        }

        let orchestrator = self.orchestrator.clone();

        match mode {
            InputMode::Url => {
                let url = text.to_string();
                self.navigate(&url, cx);
                self.set_status(format!("Navigating to {}", text), cx);
                self.sidebar.update(cx, |sb, cx| {
                    sb.add_action(SharedString::from(format!("URL: {}", text)), cx);
                });
            }
            InputMode::Transaction => {
                let intent = orchestrator.parse_input_sync(&text);
                if let Intent::Transact { to, amount, token } = &intent {
                    let wallet_e = self.wallet.clone();
                    let tx_result = wallet_e.read_with(cx, |w, _| {
                        w.build_transaction(to, amount, token)
                    });
                    match tx_result {
                        Ok(tx) => {
                            self.tx_stage = Some(TxStage::Confirm(tx));
                            self.set_status("Review transaction", cx);
                        }
                        Err(e) => {
                            self.set_status(format!("TX error: {}", e), cx);
                        }
                    }
                }
                self.sidebar.update(cx, |sb, cx| {
                    sb.add_action(SharedString::from(format!("TX: {}", text)), cx);
                });
            }
            InputMode::NaturalLanguage | InputMode::Unknown => {
                let input = text.to_string();
                self.set_status(
                    format!("Thinking: {}", &input[..input.len().min(60)]),
                    cx,
                );

                let mut c = cx.to_async();
                cx.spawn(|this: gpui::WeakEntity<Self>, _cx: &mut gpui::AsyncApp| async move {
                    let intent = orchestrator.classify_with_llm(&input).await;
                    this.update(&mut c, |s, cx| {
                        let action_text = &input[..input.len().min(60)];
                        match intent {
                            Ok(intent) => {
                                let route = s.orchestrator.route(&intent);
                                s.set_status(route, cx);
                                s.sidebar.update(cx, |sb, cx| {
                                    sb.add_action(
                                        SharedString::from(format!("AI: {}", action_text)),
                                        cx,
                                    );
                                });
                                if let Intent::Navigate { url } = &intent {
                                    s.navigate(url, cx);
                                }
                                if let Intent::Transact { to, amount, token } = &intent {
                                    let tx_result = s.wallet.read_with(cx, |w, _| {
                                        w.build_transaction(to, amount, token)
                                    });
                                    match tx_result {
                                        Ok(tx) => s.tx_stage = Some(TxStage::Confirm(tx)),
                                        Err(e) => s.set_status(format!("TX error: {}", e), cx),
                                    }
                                }
                            }
                            Err(_) => {
                                let fallback = s.orchestrator.parse_input_sync(&input);
                                let route = s.orchestrator.route(&fallback);
                                s.set_status(route, cx);
                                s.sidebar.update(cx, |sb, cx| {
                                    sb.add_action(
                                        SharedString::from(format!("Search: {}", action_text)),
                                        cx,
                                    );
                                });
                                if let Intent::Navigate { url } = &fallback {
                                    s.navigate(url, cx);
                                }
                            }
                        }
                    }).ok();
                    anyhow::Ok(())
                }).detach();
            }
        }
    }

    fn confirm_transaction(&mut self, cx: &mut Context<Self>) {
        let stage = self.tx_stage.take();
        let tx = match stage {
            Some(TxStage::Confirm(tx)) => tx,
            _ => return,
        };

        self.tx_stage = Some(TxStage::Signing);
        self.set_status("Signing transaction...", cx);

        let wallet_e = self.wallet.clone();
        let mut c = cx.to_async();
        cx.spawn(|this: gpui::WeakEntity<Self>, _cx: &mut gpui::AsyncApp| async move {
            // Simulate signing: hash the tx details and sign the hash
            let tx_hash = {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(tx.from.as_bytes());
                h.update(tx.to.as_bytes());
                h.update(tx.amount.as_bytes());
                h.update(tx.token.as_bytes());
                h.finalize()
            };

            let result = wallet_e.read_with(&c, |w, _| w.sign_transaction_hash(&tx_hash.into()));

            this.update(&mut c, |s, cx| match result {
                Ok(_sig) => {
                    let tx_hex = hex::encode(&tx_hash[..]);
                    let short = if tx_hex.len() > 20 {
                        format!("0x{}...", &tx_hex[..16])
                    } else {
                        format!("0x{}", tx_hex)
                    };
                    s.tx_stage = Some(TxStage::Signed {
                        tx_hash: SharedString::from(short),
                    });
                    s.set_status("Transaction signed", cx);
                    s.sidebar.update(cx, |sb, cx| {
                        sb.add_action(
                            SharedString::from(format!("Signed: {} {} {} → {}",
                                tx.amount, tx.token, tx.from.chars().take(10).collect::<String>(),
                                tx.to.chars().take(10).collect::<String>())),
                            cx,
                        );
                    });
                }
                Err(e) => {
                    s.tx_stage = Some(TxStage::Failed(SharedString::from(format!("{}", e))));
                    s.set_status("Transaction failed", cx);
                }
            }).ok();
            anyhow::Ok(())
        }).detach();
    }

    fn cancel_transaction(&mut self, cx: &mut Context<Self>) {
        self.tx_stage = None;
        self.set_status("Transaction cancelled", cx);
    }

    fn navigate(&mut self, url: &str, cx: &mut Context<Self>) {
        let url = if !url.contains("://") {
            format!("https://{}", url)
        } else {
            url.to_string()
        };
        let url_shared = SharedString::from(url.clone());
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.url = url_shared.clone();
            tab.title = url_shared.clone();
            tab.push_history(url_shared);
            if let Some(ref wv) = tab.webview {
                let _ = wv.load_url(&url);
            }
        }
        cx.notify();
    }

    fn go_back(&mut self, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else {
            return;
        };
        if !tab.can_go_back() {
            return;
        }
        tab.history_index -= 1;
        let url = tab.history[tab.history_index].clone();
        tab.url = url.clone();
        tab.title = url.clone();
        if let Some(ref wv) = tab.webview {
            let _ = wv.load_url(&url);
        }
        cx.notify();
    }

    fn go_forward(&mut self, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else {
            return;
        };
        if !tab.can_go_forward() {
            return;
        }
        tab.history_index += 1;
        let url = tab.history[tab.history_index].clone();
        tab.url = url.clone();
        tab.title = url.clone();
        if let Some(ref wv) = tab.webview {
            let _ = wv.load_url(&url);
        }
        cx.notify();
    }

    fn reload_active(&mut self, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if let Some(ref wv) = tab.webview {
                let _ = wv.evaluate_script("location.reload()");
            }
        }
        cx.notify();
    }

    fn set_status(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.status_text = text.into();
        cx.notify();
    }

    fn new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let url = "https://matterhorn.so";
        let webview = wry::WebViewBuilder::new()
            .with_url(url)
            .with_bounds(wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
                size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(800.0, 600.0)),
            })
            .build_as_child(window)
            .ok();

        let mut tab = Tab::new("New Tab", url);
        tab.webview = webview;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        cx.notify();
    }

    /// Resize the active tab's webview to match current window bounds.
    /// Idempotent — skips the IPC to wry if bounds haven't changed since the
    /// last sync. Safe to call from render().
    fn sync_active_webview_bounds(&mut self, window: &mut Window) {
        let bounds = window.bounds();
        let active = self.active_tab;
        let Some(tab) = self.tabs.get_mut(active) else {
            return;
        };
        if tab.last_bounds == bounds {
            return;
        }
        tab.last_bounds = bounds;
        let sidebar_w: f32 = if self.sidebar_visible { 280.0 } else { 0.0 };
        let y_offset = 72.0_f32;
        let width = bounds.size.width.as_f32() - sidebar_w;
        let height = bounds.size.height.as_f32() - y_offset;
        if let Some(ref wv) = tab.webview {
            let _ = wv.set_bounds(wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                    0.0,
                    y_offset.into(),
                )),
                size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                    width.max(100.0).into(),
                    height.max(100.0).into(),
                )),
            });
        }
    }

    fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        cx.notify();
    }

    fn close_active_tab(&mut self, cx: &mut Context<Self>) {
        let idx = self.active_tab;
        self.close_tab(idx, cx);
    }

    fn next_tab(&mut self, cx: &mut Context<Self>) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            cx.notify();
        }
    }

    fn prev_tab(&mut self, cx: &mut Context<Self>) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + self.tabs.len() - 1) % self.tabs.len();
            cx.notify();
        }
    }

    fn render_confirmation_sheet(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let sheet = match &self.tx_stage {
            Some(TxStage::Confirm(tx)) => div()
                .flex_col()
                .p_4()
                .gap_3()
                .bg(rgb(SURFACE))
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .child(
                    div()
                        .text_lg()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(TEXT))
                        .child("Confirm Transaction"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_MUTED))
                        .child(format!(
                            "Send {} {} from {} to {}",
                            tx.amount,
                            tx.token,
                            short_addr(&tx.from),
                            short_addr(&tx.to)
                        )),
                )
                .child(
                    div()
                        .flex_row()
                        .gap_2()
                        .child(
                            div()
                                .id("tx-cancel")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(SURFACE_ALT))
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .cursor_pointer()
                                .hover(|el| el.bg(rgb(BORDER)))
                                .on_click(cx.listener(
                                    |this: &mut Self,
                                     _: &gpui::ClickEvent,
                                     _: &mut Window,
                                     cx: &mut Context<Self>| {
                                        this.cancel_transaction(cx);
                                    },
                                ))
                                .child("Cancel"),
                        )
                        .child(
                            div()
                                .id("tx-confirm")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(ACCENT))
                                .text_sm()
                                .text_color(rgb(BG))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(|el| el.opacity(0.8))
                                .on_click(cx.listener(
                                    |this: &mut Self,
                                     _: &gpui::ClickEvent,
                                     _: &mut Window,
                                     cx: &mut Context<Self>| {
                                        this.confirm_transaction(cx);
                                    },
                                ))
                                .child("Sign & Send"),
                        ),
                ),
            Some(TxStage::Signing) => div()
                .flex_col()
                .p_4()
                .gap_3()
                .bg(rgb(SURFACE))
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(TEXT))
                        .child("Signing transaction..."),
                ),
            Some(TxStage::Signed { tx_hash }) => div()
                .flex_col()
                .p_4()
                .gap_3()
                .bg(rgb(SURFACE))
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(GREEN))
                        .child("\u{2713} Transaction Signed"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(TEXT_MUTED))
                        .child(format!("TX Hash: {}", tx_hash)),
                )
                .child(
                    div()
                        .id("tx-dismiss")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(rgb(SURFACE_ALT))
                        .text_sm()
                        .text_color(rgb(TEXT_MUTED))
                        .cursor_pointer()
                        .hover(|el| el.bg(rgb(BORDER)))
                        .on_click(cx.listener(
                            |this: &mut Self,
                             _: &gpui::ClickEvent,
                             _: &mut Window,
                             cx: &mut Context<Self>| {
                                this.tx_stage = None;
                                cx.notify();
                            },
                        ))
                        .child("Dismiss"),
                ),
            Some(TxStage::Failed(msg)) => div()
                .flex_col()
                .p_4()
                .gap_3()
                .bg(rgb(SURFACE))
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(RED))
                        .child("Transaction Failed"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_MUTED))
                        .child(msg.clone()),
                )
                .child(
                    div()
                        .id("tx-dismiss")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(rgb(SURFACE_ALT))
                        .text_sm()
                        .text_color(rgb(TEXT_MUTED))
                        .cursor_pointer()
                        .hover(|el| el.bg(rgb(BORDER)))
                        .on_click(cx.listener(
                            |this: &mut Self,
                             _: &gpui::ClickEvent,
                             _: &mut Window,
                             cx: &mut Context<Self>| {
                                this.tx_stage = None;
                                cx.notify();
                            },
                        ))
                        .child("Dismiss"),
                ),
            None => return div(),
        };

        div()
            .absolute()
            .inset_0()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(gpui::rgba(0x00000066))
            .child(sheet)
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let many_tabs = self.tabs.len() > 1;
        div()
            .flex_row()
            .items_center()
            .bg(rgb(SURFACE_ALT))
            .border_b_1()
            .border_color(rgb(BORDER))
            .children(self.tabs.iter().enumerate().map(|(i, tab)| {
                let is_active = i == self.active_tab;
                let tab_id = format!("tab-{}", i);
                let close_id = format!("tab-close-{}", i);
                div()
                    .id(tab_id)
                    .flex_row()
                    .items_center()
                    .px_3()
                    .py_1p5()
                    .gap_2()
                    .text_sm()
                    .when(is_active, |el| {
                        el.bg(rgb(SURFACE)).border_b_2().border_color(rgb(ACCENT))
                    })
                    .when(!is_active, |el| {
                        el.text_color(rgb(TEXT_MUTED))
                            .cursor_pointer()
                            .hover(|el| el.bg(rgb(SURFACE)))
                    })
                    .on_click(cx.listener(
                        move |this: &mut Self,
                              _: &gpui::ClickEvent,
                              window: &mut Window,
                              cx: &mut Context<Self>| {
                            this.set_active_tab(i, window, cx);
                        },
                    ))
                    .child(tab.title.clone())
                    .when(many_tabs, |el| {
                        el.child(
                            div()
                                .id(close_id)
                                .px_1()
                                .text_xs()
                                .text_color(rgb(TEXT_DIM))
                                .cursor_pointer()
                                .hover(|el| el.text_color(rgb(TEXT)))
                                .on_click(cx.listener(
                                    move |this: &mut Self,
                                          _: &gpui::ClickEvent,
                                          window: &mut Window,
                                          cx: &mut Context<Self>| {
                                        this.close_tab(i, cx);
                                        this.sync_active_webview_bounds(window);
                                    },
                                ))
                                .child("x"),
                        )
                    })
                    .into_any_element()
            }))
            .child(
                div()
                    .id("new-tab-btn")
                    .px_3()
                    .text_sm()
                    .text_color(rgb(TEXT_MUTED))
                    .cursor_pointer()
                    .hover(|el| el.text_color(rgb(TEXT)))
                    .on_click(cx.listener(
                        |this: &mut Self,
                         _: &gpui::ClickEvent,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                            this.new_tab(window, cx);
                            this.sync_active_webview_bounds(window);
                        },
                    ))
                    .child("+"),
            )
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.tabs.get(self.active_tab);
        let can_back = active.map(|t| t.can_go_back()).unwrap_or(false);
        let can_forward = active.map(|t| t.can_go_forward()).unwrap_or(false);

        let back_color = if can_back {
            rgb(TEXT)
        } else {
            rgb(TEXT_DIM)
        };
        let forward_color = if can_forward {
            rgb(TEXT)
        } else {
            rgb(TEXT_DIM)
        };

        div()
            .flex_row()
            .items_center()
            .px_2()
            .py_1()
            .gap_2()
            .bg(rgb(SURFACE))
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .id("nav-back")
                            .px_2()
                            .text_sm()
                            .text_color(back_color)
                            .when(can_back, |el| {
                                el.cursor_pointer().hover(|el| el.text_color(rgb(ACCENT)))
                            })
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &gpui::ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.go_back(cx);
                                },
                            ))
                            .child("\u{2190}"),
                    )
                    .child(
                        div()
                            .id("nav-forward")
                            .px_2()
                            .text_sm()
                            .text_color(forward_color)
                            .when(can_forward, |el| {
                                el.cursor_pointer().hover(|el| el.text_color(rgb(ACCENT)))
                            })
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &gpui::ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.go_forward(cx);
                                },
                            ))
                            .child("\u{2192}"),
                    )
                    .child(
                        div()
                            .id("nav-reload")
                            .px_2()
                            .text_sm()
                            .text_color(rgb(TEXT_MUTED))
                            .cursor_pointer()
                            .hover(|el| el.text_color(rgb(ACCENT)))
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &gpui::ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.reload_active(cx);
                                },
                            ))
                            .child("\u{21BB}"),
                    ),
            )
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_MUTED))
                            .child(format!("ETH: {}", self.eth_balance)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_MUTED))
                            .child(format!("SOL: {}", self.sol_balance)),
                    )
                    .when(self.ens_name.as_ref() != "...", |el| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(rgb(GREEN))
                                .child(self.ens_name.clone()),
                        )
                    }),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(TEXT_DIM))
                    .child(self.status_text.clone()),
            )
    }

    fn set_active_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.active_tab = index;
        self.sync_active_webview_bounds(window);
        cx.notify();
    }
}

/// Truncate an address to a 10-char prefix for display ("0xabc...").
fn short_addr(addr: &str) -> String {
    if addr.len() <= 10 {
        addr.to_string()
    } else {
        format!("{}...", &addr[..addr.len().min(10)])
    }
}

impl Render for BrowserState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Onboarding / unlock screens render the OnboardingState entity full-screen.
        if matches!(self.phase, BrowserPhase::Onboarding | BrowserPhase::Unlocking) {
            return div()
                .size_full()
                .bg(rgb(BG))
                .child(self.onboarding.clone())
                .into_any_element();
        }

        // Browsing phase: ensure the active tab has a WebView, then sync bounds.
        // set_bounds is idempotent — calling it from render is safe and does not
        // trigger a re-render via cx.notify().
        self.ensure_webview(window, cx);
        self.sync_active_webview_bounds(window);

        let composer = self.composer.clone();
        let sidebar_open = self.sidebar_visible;
        let has_tx_sheet = self.tx_stage.is_some();

        div()
            .relative()
            .size_full()
            .flex_col()
            .bg(rgb(BG))
            .font_family("System-ui")
            .on_key_down(cx.listener(
                |this: &mut Self,
                 ev: &KeyDownEvent,
                 window: &mut Window,
                 cx: &mut Context<Self>| {
                    if this.tx_stage.is_some() {
                        match ev.keystroke.key.as_str() {
                            "escape" => this.cancel_transaction(cx),
                            "enter" | "return" => {
                                if matches!(this.tx_stage, Some(TxStage::Confirm(_))) {
                                    this.confirm_transaction(cx);
                                } else {
                                    this.tx_stage = None;
                                    cx.notify();
                                }
                            }
                            _ => {}
                        }
                        return;
                    }
                    let meta = ev.keystroke.modifiers.platform;
                    match ev.keystroke.key.as_str() {
                        "t" if meta => {
                            this.new_tab(window, cx);
                            this.sync_active_webview_bounds(window);
                        }
                        "w" if meta => {
                            this.close_active_tab(cx);
                            this.sync_active_webview_bounds(window);
                        }
                        "l" if meta => this.composer.focus_handle(cx).focus(window, cx),
                        "k" if meta => {
                            this.composer.update(cx, |c, cx| c.show_command_palette(cx));
                        }
                        "b" if meta => {
                            this.sidebar_visible = !this.sidebar_visible;
                            this.sync_active_webview_bounds(window);
                            cx.notify();
                        }
                        "[" if meta => {
                            this.prev_tab(cx);
                            this.sync_active_webview_bounds(window);
                        }
                        "]" if meta => {
                            this.next_tab(cx);
                            this.sync_active_webview_bounds(window);
                        }
                        "r" if meta => this.reload_active(cx),
                        _ => {}
                    }
                },
            ))
            .child(self.render_tab_bar(cx))
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .flex_row()
                    .flex_1()
                    .relative()
                    .child(div().id("viewport").flex_1().bg(gpui::rgba(0xFFFFFFFF)))
                    .when(sidebar_open, |el| {
                        el.child(
                            div()
                                .w(px(280.0))
                                .h_full()
                                .bg(rgb(SURFACE))
                                .border_l_1()
                                .border_color(rgb(BORDER))
                                .child(self.sidebar.clone()),
                        )
                    }),
            )
            .child(composer)
            .when(has_tx_sheet, |el| {
                el.child(self.render_confirmation_sheet(window, cx))
            })
            .into_any_element()
    }
}
