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
    Unlock,
    Complete,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ImportField {
    Phrase,
    Password,
}

pub struct OnboardingState {
    pub step: OnboardingStep,
    pub wallet: Entity<MatterhornWallet>,
    pub done: bool,
    pub password_input: SharedString,
    pub phrase_input: SharedString,
    pub import_field: ImportField,
    pub show_password: bool,
    pub error: SharedString,
}

impl OnboardingState {
    pub fn new(_cx: &mut Context<Self>, wallet: Entity<MatterhornWallet>) -> Self {
        Self {
            step: OnboardingStep::Welcome,
            wallet,
            done: false,
            password_input: SharedString::from(""),
            phrase_input: SharedString::from(""),
            import_field: ImportField::Phrase,
            show_password: false,
            error: SharedString::from(""),
        }
    }

    /// Construct the onboarding state in unlock mode (used when a wallet is
    /// already stored in the keychain and the user just needs to unlock it).
    pub fn unlocking(_cx: &mut Context<Self>, wallet: Entity<MatterhornWallet>) -> Self {
        Self {
            step: OnboardingStep::Unlock,
            wallet,
            done: false,
            password_input: SharedString::from(""),
            phrase_input: SharedString::from(""),
            import_field: ImportField::Password,
            show_password: false,
            error: SharedString::from(""),
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
            OnboardingStep::Unlock => self.render_unlock(cx).into_any_element(),
            OnboardingStep::Complete => self.render_complete(cx).into_any_element(),
        }
    }
}

/// Render a password as a string of bullets, preserving length so the user
/// gets feedback on what they've typed without exposing the plaintext.
fn mask_password(input: &str, show: bool) -> String {
    if show {
        input.to_string()
    } else {
        "\u{2022}".repeat(input.chars().count())
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
            "(type your password)".to_string()
        } else {
            mask_password(&self.password_input, self.show_password)
        };
        let toggle_label = if self.show_password { "hide" } else { "show" };

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
                        let password = this.password_input.to_string();
                        if password.is_empty() {
                            return;
                        }
                        this.wallet.update(cx, |wallet, _cx| {
                            let _ = wallet.create(&password);
                            let _ = wallet.create_solana(&password);
                        });
                        this.go_to(OnboardingStep::Complete, cx);
                    }
                    "space" => {
                        this.password_input = SharedString::from(format!("{} ", this.password_input));
                        cx.notify();
                    }
                    key if key.len() == 1 && !ev.keystroke.modifiers.control && !ev.keystroke.modifiers.platform => {
                        this.password_input = SharedString::from(format!("{}{}", this.password_input, key));
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
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .w(px(260.0))
                                    .rounded_md()
                                    .bg(rgb(SURFACE))
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .text_sm()
                                    .text_color(rgb(TEXT))
                                    .child(password_display),
                            )
                            .child(
                                div()
                                    .id("toggle-show-password-create")
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .cursor_pointer()
                                    .hover(|el| el.text_color(rgb(TEXT)))
                                    .on_click(cx.listener(
                                        |this: &mut Self,
                                         _: &ClickEvent,
                                         _: &mut Window,
                                         cx: &mut Context<Self>| {
                                            this.show_password = !this.show_password;
                                            cx.notify();
                                        },
                                    ))
                                    .child(toggle_label.to_string()),
                            ),
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
        let phrase_display = if self.phrase_input.is_empty() {
            "(paste your 12 or 24-word seed phrase)".to_string()
        } else {
            self.phrase_input.to_string()
        };
        let password_display = if self.password_input.is_empty() {
            "(type a password)".to_string()
        } else {
            mask_password(&self.password_input, self.show_password)
        };
        let toggle_label = if self.show_password { "hide" } else { "show" };
        let phrase_active = self.import_field == ImportField::Phrase;
        let password_active = self.import_field == ImportField::Password;
        let error = self.error.clone();
        let has_error = !error.is_empty();

        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .on_key_down(cx.listener(|this: &mut Self, ev: &gpui::KeyDownEvent, _window, cx| {
                let key = ev.keystroke.key.as_str();
                let mods = &ev.keystroke.modifiers;
                match key {
                    "tab" => {
                        this.import_field = match this.import_field {
                            ImportField::Phrase => ImportField::Password,
                            ImportField::Password => ImportField::Phrase,
                        };
                        cx.notify();
                    }
                    "backspace" => {
                        match this.import_field {
                            ImportField::Phrase => {
                                let mut s = this.phrase_input.to_string();
                                s.pop();
                                this.phrase_input = SharedString::from(s);
                            }
                            ImportField::Password => {
                                let mut s = this.password_input.to_string();
                                s.pop();
                                this.password_input = SharedString::from(s);
                            }
                        }
                        cx.notify();
                    }
                    "enter" | "return" => {
                        this.try_import(cx);
                    }
                    "space" => {
                        match this.import_field {
                            ImportField::Phrase => {
                                this.phrase_input =
                                    SharedString::from(format!("{} ", this.phrase_input));
                            }
                            ImportField::Password => {
                                this.password_input =
                                    SharedString::from(format!("{} ", this.password_input));
                            }
                        }
                        cx.notify();
                    }
                    key if key.len() == 1 && !mods.control && !mods.platform => {
                        match this.import_field {
                            ImportField::Phrase => {
                                this.phrase_input =
                                    SharedString::from(format!("{}{}", this.phrase_input, key));
                            }
                            ImportField::Password => {
                                this.password_input =
                                    SharedString::from(format!("{}{}", this.password_input, key));
                            }
                        }
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            .child(
                div()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("Import Wallet"),
                    )
                    .child(
                        div().text_sm().text_color(rgb(TEXT_MUTED)).child(
                            "Paste your 12 or 24-word seed phrase, then set a password. Tab switches fields.",
                        ),
                    )
                    .child(
                        div()
                            .id("phrase-field")
                            .px_4()
                            .py_3()
                            .w(px(360.0))
                            .h(px(72.0))
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(if phrase_active {
                                rgb(ACCENT)
                            } else {
                                rgb(BORDER)
                            })
                            .text_sm()
                            .text_color(if self.phrase_input.is_empty() {
                                rgb(TEXT_MUTED)
                            } else {
                                rgb(TEXT)
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(
                                |this: &mut Self,
                                 _: &ClickEvent,
                                 _: &mut Window,
                                 cx: &mut Context<Self>| {
                                    this.import_field = ImportField::Phrase;
                                    cx.notify();
                                },
                            ))
                            .child(phrase_display),
                    )
                    .child(
                        div()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("password-field")
                                    .px_4()
                                    .py_2()
                                    .w(px(320.0))
                                    .rounded_md()
                                    .bg(rgb(SURFACE))
                                    .border_1()
                                    .border_color(if password_active {
                                        rgb(ACCENT)
                                    } else {
                                        rgb(BORDER)
                                    })
                                    .text_sm()
                                    .text_color(if self.password_input.is_empty() {
                                        rgb(TEXT_MUTED)
                                    } else {
                                        rgb(TEXT)
                                    })
                                    .cursor_pointer()
                                    .on_click(cx.listener(
                                        |this: &mut Self,
                                         _: &ClickEvent,
                                         _: &mut Window,
                                         cx: &mut Context<Self>| {
                                            this.import_field = ImportField::Password;
                                            cx.notify();
                                        },
                                    ))
                                    .child(password_display),
                            )
                            .child(
                                div()
                                    .id("toggle-show-password-import")
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .cursor_pointer()
                                    .hover(|el| el.text_color(rgb(TEXT)))
                                    .on_click(cx.listener(
                                        |this: &mut Self,
                                         _: &ClickEvent,
                                         _: &mut Window,
                                         cx: &mut Context<Self>| {
                                            this.show_password = !this.show_password;
                                            cx.notify();
                                        },
                                    ))
                                    .child(toggle_label.to_string()),
                            ),
                    )
                    .when(has_error, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(gpui::rgb(0xFF453A))
                                .child(error.clone()),
                        )
                    })
                    .child(
                        div()
                            .id("import-btn")
                            .mt_2()
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
                                    this.try_import(cx);
                                },
                            ))
                            .child("Import"),
                    ),
            )
    }

    fn try_import(&mut self, cx: &mut Context<Self>) {
        let phrase = self.phrase_input.to_string().trim().to_string();
        let password = self.password_input.to_string();
        if phrase.is_empty() {
            self.error = SharedString::from("Seed phrase is required.");
            cx.notify();
            return;
        }
        if password.is_empty() {
            self.error = SharedString::from("Password is required.");
            cx.notify();
            return;
        }
        let result = self.wallet.update(cx, |wallet, _cx| {
            wallet
                .import(&phrase, &password)
                .and_then(|_| wallet.create_solana(&password).map(|_| ()))
        });
        match result {
            Ok(_) => {
                self.error = SharedString::from("");
                self.go_to(OnboardingStep::Complete, cx);
            }
            Err(e) => {
                self.error = SharedString::from(format!("Import failed: {e}"));
                cx.notify();
            }
        }
    }

    fn render_unlock(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let password_display = if self.password_input.is_empty() {
            "(enter your password)".to_string()
        } else {
            mask_password(&self.password_input, self.show_password)
        };
        let toggle_label = if self.show_password { "hide" } else { "show" };
        let error = self.error.clone();
        let has_error = !error.is_empty();

        div()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .bg(rgb(BG))
            .on_key_down(cx.listener(|this: &mut Self, ev: &gpui::KeyDownEvent, _window, cx| {
                let key = ev.keystroke.key.as_str();
                let mods = &ev.keystroke.modifiers;
                match key {
                    "backspace" => {
                        let mut s = this.password_input.to_string();
                        s.pop();
                        this.password_input = SharedString::from(s);
                        cx.notify();
                    }
                    "enter" | "return" => {
                        this.try_unlock(cx);
                    }
                    "space" => {
                        this.password_input =
                            SharedString::from(format!("{} ", this.password_input));
                        cx.notify();
                    }
                    key if key.len() == 1 && !mods.control && !mods.platform => {
                        this.password_input =
                            SharedString::from(format!("{}{}", this.password_input, key));
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
                            .child("Unlock Wallet"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(TEXT_MUTED))
                            .child("Enter the password you set when creating this wallet."),
                    )
                    .child(
                        div()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .w(px(260.0))
                                    .rounded_md()
                                    .bg(rgb(SURFACE))
                                    .border_1()
                                    .border_color(rgb(ACCENT))
                                    .text_sm()
                                    .text_color(rgb(TEXT))
                                    .child(password_display),
                            )
                            .child(
                                div()
                                    .id("toggle-show-password-unlock")
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .cursor_pointer()
                                    .hover(|el| el.text_color(rgb(TEXT)))
                                    .on_click(cx.listener(
                                        |this: &mut Self,
                                         _: &ClickEvent,
                                         _: &mut Window,
                                         cx: &mut Context<Self>| {
                                            this.show_password = !this.show_password;
                                            cx.notify();
                                        },
                                    ))
                                    .child(toggle_label.to_string()),
                            ),
                    )
                    .when(has_error, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(gpui::rgb(0xFF453A))
                                .child(error.clone()),
                        )
                    })
                    .child(
                        div()
                            .id("unlock-btn")
                            .mt_2()
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
                                    this.try_unlock(cx);
                                },
                            ))
                            .child("Unlock"),
                    ),
            )
    }

    fn try_unlock(&mut self, cx: &mut Context<Self>) {
        let password = self.password_input.to_string();
        if password.is_empty() {
            self.error = SharedString::from("Password is required.");
            cx.notify();
            return;
        }
        let result = self
            .wallet
            .update(cx, |wallet, _cx| wallet.load_from_keychain(&password));
        match result {
            Ok(_) => {
                self.error = SharedString::from("");
                self.done = true;
                self.password_input = SharedString::from("");
                cx.notify();
            }
            Err(e) => {
                self.error = SharedString::from(format!("Unlock failed: {e}"));
                self.password_input = SharedString::from("");
                cx.notify();
            }
        }
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
