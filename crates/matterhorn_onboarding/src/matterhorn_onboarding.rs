use gpui::{
    ClickEvent, Context, Entity, FontWeight, ParentElement, Render, SharedString,
    Styled, Window, div, prelude::*, px, rgb,
};
use matterhorn_wallet::MatterhornWallet;

const BG: u32 = 0x0C0C0C;
const ACCENT: u32 = 0xD1F2FF;
const SURFACE: u32 = 0x1C1C1E;
const SURFACE_ALT: u32 = 0x161618;
const BORDER: u32 = 0x2C2C2E;
const TEXT_MUTED: u32 = 0xA1A1A6;
const TEXT: u32 = 0xFFFFFF;

#[derive(Clone, PartialEq, Debug)]
pub enum OnboardingStep {
    Welcome,
    CreateOrImport,
    CreateWallet { password: SharedString },
    ImportWallet { phrase: SharedString, password: SharedString },
    Complete,
}

pub struct OnboardingState {
    pub step: OnboardingStep,
    pub wallet: Entity<MatterhornWallet>,
    pub done: bool,
    pub password_input: SharedString,
}

impl OnboardingState {
    pub fn new(_cx: &mut Context<Self>, wallet: Entity<MatterhornWallet>) -> Self {
        Self {
            step: OnboardingStep::Welcome,
            wallet,
            done: false,
            password_input: SharedString::from(""),
        }
    }

    fn go_to(&mut self, step: OnboardingStep, cx: &mut Context<Self>) {
        self.step = step;
        cx.notify();
    }

    fn complete(&mut self, cx: &mut Context<Self>) {
        self.wallet.update(cx, |wallet, _cx| {
            if let Err(e) = wallet.store_in_keychain() {
                eprintln!("Failed to store wallet in keychain: {e}");
            }
        });
        self.done = true;
        cx.notify();
    }
}

impl Render for OnboardingState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.step {
            OnboardingStep::Welcome => self.render_welcome(cx).into_any_element(),
            OnboardingStep::CreateOrImport => self.render_create_or_import(cx).into_any_element(),
            OnboardingStep::CreateWallet { .. } => self.render_create_wallet(cx).into_any_element(),
            OnboardingStep::ImportWallet { .. } => self.render_import_wallet(cx).into_any_element(),
            OnboardingStep::Complete => self.render_complete(cx).into_any_element(),
        }
    }
}

impl OnboardingState {
    fn render_welcome(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .child(
                div()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Matterhorn Browser"),
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(TEXT_MUTED))
                            .child("Browse Web3. Natively."),
                    )
                    .child(
                        div()
                            .id("get-started-btn")
                            .mt_4()
                            .px_6()
                            .py_3()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(BG))
                            .font_weight(FontWeight::MEDIUM)
                            .cursor_pointer()
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.go_to(OnboardingStep::CreateOrImport, cx);
                                },
                            ))
                            .child("Get Started"),
                    ),
            )
    }

    fn render_create_or_import(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .child(
                div()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Set Up Your Wallet"),
                    )
                    .child(
                        div()
                            .id("create-wallet-btn")
                            .px_6()
                            .py_3()
                            .w(px(260.0))
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .cursor_pointer()
                            .hover(|el| el.bg(rgb(SURFACE_ALT)))
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.go_to(
                                        OnboardingStep::CreateWallet {
                                            password: SharedString::from(""),
                                        },
                                        cx,
                                    );
                                },
                            ))
                            .child(
                                div()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(TEXT))
                                            .child("Create New Wallet"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .child("Generate a new seed phrase"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("import-wallet-btn")
                            .px_6()
                            .py_3()
                            .w(px(260.0))
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .cursor_pointer()
                            .hover(|el| el.bg(rgb(SURFACE_ALT)))
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.go_to(
                                        OnboardingStep::ImportWallet {
                                            phrase: SharedString::from(""),
                                            password: SharedString::from(""),
                                        },
                                        cx,
                                    );
                                },
                            ))
                            .child(
                                div()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(TEXT))
                                            .child("Import Existing Wallet"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(TEXT_MUTED))
                                            .child("Restore from seed phrase"),
                                    ),
                            ),
                    ),
            )
    }

    fn render_create_wallet(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let password_display = if self.password_input.is_empty() {
            "(type your password)"
        } else {
            &self.password_input
        };

        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .on_key_down(cx.listener(|this: &mut Self, ev: &gpui::KeyDownEvent, _window, cx| {
                match ev.keystroke.key.as_str() {
                    "backspace" => {
                        let mut s = this.password_input.to_string();
                        s.pop();
                        this.password_input = SharedString::from(s);
                        cx.notify();
                    }
                    key if key.len() == 1 && !ev.keystroke.modifiers.control && !ev.keystroke.modifiers.platform => {
                        this.password_input = SharedString::from(format!("{}{}", this.password_input, key));
                        cx.notify();
                    }
                    "space" => {
                        this.password_input = SharedString::from(format!("{} ", this.password_input));
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            .child(
                div()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Create Wallet"),
                    )
                    .child(
                        div().text_sm().text_color(rgb(TEXT_MUTED)).child(
                            "Choose a password to encrypt your wallet.",
                        ),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .w(px(300.0))
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .text_color(rgb(TEXT))
                            .child(password_display.to_string()),
                    )
                    .child(
                        div()
                            .id("create-btn")
                            .mt_4()
                            .px_6()
                            .py_3()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(BG))
                            .font_weight(FontWeight::MEDIUM)
                            .cursor_pointer()
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    let password = this.password_input.to_string();
                                    if password.is_empty() {
                                        return;
                                    }
                                    this.wallet.update(cx, |wallet, _cx| {
                                        let _ = wallet.create(&password);
                                        let _ = wallet.create_solana(&password);
                                    });
                                    this.go_to(OnboardingStep::Complete, cx);
                                },
                            ))
                            .child("Create Wallet"),
                    ),
            )
    }

    fn render_import_wallet(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let password_display = if self.password_input.is_empty() {
            "(type your password)"
        } else {
            &self.password_input
        };

        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .on_key_down(cx.listener(|this: &mut Self, ev: &gpui::KeyDownEvent, _window, cx| {
                match ev.keystroke.key.as_str() {
                    "backspace" => {
                        let mut s = this.password_input.to_string();
                        s.pop();
                        this.password_input = SharedString::from(s);
                        cx.notify();
                    }
                    "enter" | "return" => {
                        // use Enter to confirm import
                        let password = this.password_input.to_string();
                        let phrase = this.password_input.to_string();
                        if password.is_empty() {
                            return;
                        }
                        this.wallet.update(cx, |wallet, _cx| {
                            let _ = wallet.import(&phrase, &password);
                            let _ = wallet.create_solana(&password);
                        });
                        this.go_to(OnboardingStep::Complete, cx);
                    }
                    key if key.len() == 1 && !ev.keystroke.modifiers.control && !ev.keystroke.modifiers.platform => {
                        this.password_input = SharedString::from(format!("{}{}", this.password_input, key));
                        cx.notify();
                    }
                    "space" => {
                        this.password_input = SharedString::from(format!("{} ", this.password_input));
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            .child(
                div()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Import Wallet"),
                    )
                    .child(
                        div().text_sm().text_color(rgb(TEXT_MUTED)).child(
                            "Paste your 12 or 24-word seed phrase, then set a password.",
                        ),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .w(px(300.0))
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .text_color(rgb(TEXT))
                            .child(password_display.to_string()),
                    )
                    .child(
                        div()
                            .id("import-btn")
                            .mt_4()
                            .px_6()
                            .py_3()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(BG))
                            .font_weight(FontWeight::MEDIUM)
                            .cursor_pointer()
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    let password = this.password_input.to_string();
                                    let phrase = this.password_input.to_string();
                                    if password.is_empty() {
                                        return;
                                    }
                                    this.wallet.update(cx, |wallet, _cx| {
                                        let _ = wallet.import(&phrase, &password);
                                        let _ = wallet.create_solana(&password);
                                    });
                                    this.go_to(OnboardingStep::Complete, cx);
                                },
                            ))
                            .child("Import"),
                    ),
            )
    }

    fn render_complete(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let address = self
            .wallet
            .read(cx)
            .selected_address()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "Unknown".into());

        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .child(
                div()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Wallet Ready"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(TEXT_MUTED))
                            .child("Your wallet has been set up."),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT))
                            .child(format!("Address: {}...", &address[..address.len().min(20)])),
                    )
                    .child(
                        div()
                            .id("start-browsing-btn")
                            .mt_4()
                            .px_6()
                            .py_3()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(BG))
                            .font_weight(FontWeight::MEDIUM)
                            .cursor_pointer()
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.complete(cx);
                                },
                            ))
                            .child("Start Browsing"),
                    ),
            )
    }
}
