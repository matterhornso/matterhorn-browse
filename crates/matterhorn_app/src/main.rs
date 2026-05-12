// Matterhorn App — spike of Option B from REVIEW.md.
//
// Goal: prove GPUI renders text against our brand colors when used as a
// library, with zero Zed-fork scaffolding. Mirrors the structure of
// crates/gpui/examples/hello_world.rs — application() entry, single Render
// implementation, no settings/theme/font-asset wiring.
//
// If this binary opens a window with visible "Matterhorn Browser" text on
// the dark background, the diagnosis in REVIEW.md is confirmed and we
// proceed to port the eight matterhorn_* crates into this clean tree.

use gpui::{
    App, Bounds, ClickEvent, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use gpui_platform::application;

// Matterhorn brand colors, same as the legacy crates use.
const BG: u32 = 0x0C0C0C;
const ACCENT: u32 = 0xD1F2FF;
const SURFACE: u32 = 0x1C1C1E;
const BORDER: u32 = 0x2C2C2E;
const TEXT: u32 = 0xFFFFFF;
const TEXT_MUTED: u32 = 0xA1A1A6;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Step {
    Welcome,
    CreateOrImport,
}

struct App2 {
    step: Step,
    click_count: u32,
}

impl App2 {
    fn new() -> Self {
        Self {
            step: Step::Welcome,
            click_count: 0,
        }
    }

    fn go_to(&mut self, step: Step, cx: &mut Context<Self>) {
        self.step = step;
        cx.notify();
    }
}

impl Render for App2 {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body: gpui::AnyElement = match self.step {
            Step::Welcome => render_welcome(cx).into_any_element(),
            Step::CreateOrImport => render_create_or_import(cx, self.click_count).into_any_element(),
        };
        div()
            .size_full()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .child(body)
    }
}

fn render_welcome(cx: &mut Context<App2>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap_6()
        .child(
            div()
                .text_3xl()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(ACCENT))
                .child(SharedString::from("Matterhorn Browser")),
        )
        .child(
            div()
                .text_lg()
                .text_color(rgb(TEXT_MUTED))
                .child(SharedString::from("Browse Web3. Natively.")),
        )
        .child(
            div()
                .id("get-started-btn")
                .px_6()
                .py_3()
                .rounded_md()
                .bg(rgb(ACCENT))
                .text_color(rgb(BG))
                .font_weight(FontWeight::MEDIUM)
                .cursor_pointer()
                .hover(|el| el.opacity(0.85))
                .on_click(cx.listener(
                    |this: &mut App2, _: &ClickEvent, _: &mut Window, cx: &mut Context<App2>| {
                        this.go_to(Step::CreateOrImport, cx);
                    },
                ))
                .child(SharedString::from("Get Started")),
        )
}

fn render_create_or_import(cx: &mut Context<App2>, click_count: u32) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap_6()
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(ACCENT))
                .child(SharedString::from("Set Up Your Wallet")),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .child(SharedString::from(format!(
                    "Spike round-trip OK. Clicks: {}",
                    click_count
                ))),
        )
        .child(
            div()
                .flex_col()
                .gap_3()
                .child(option_card(
                    cx,
                    "create-wallet-btn",
                    "Create New Wallet",
                    "Generate a new seed phrase",
                ))
                .child(option_card(
                    cx,
                    "import-wallet-btn",
                    "Import Existing Wallet",
                    "Restore from a 12 or 24 word phrase",
                )),
        )
        .child(
            div()
                .id("back-btn")
                .px_4()
                .py_2()
                .text_sm()
                .text_color(rgb(TEXT_MUTED))
                .cursor_pointer()
                .hover(|el| el.text_color(rgb(TEXT)))
                .on_click(cx.listener(
                    |this: &mut App2, _: &ClickEvent, _: &mut Window, cx: &mut Context<App2>| {
                        this.click_count = this.click_count.wrapping_add(1);
                        this.go_to(Step::Welcome, cx);
                    },
                ))
                .child(SharedString::from("\u{2190} Back")),
        )
}

fn option_card(
    cx: &mut Context<App2>,
    id: &'static str,
    title: &'static str,
    sub: &'static str,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(320.0))
        .px_5()
        .py_3()
        .rounded_md()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(BORDER))
        .cursor_pointer()
        .hover(|el| el.border_color(rgb(ACCENT)))
        .on_click(cx.listener(
            |this: &mut App2, _: &ClickEvent, _: &mut Window, cx: &mut Context<App2>| {
                this.click_count = this.click_count.wrapping_add(1);
                cx.notify();
            },
        ))
        .child(
            div()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(TEXT))
                        .child(SharedString::from(title)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(TEXT_MUTED))
                        .child(SharedString::from(sub)),
                ),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| App2::new()),
        )
        .unwrap();
        cx.activate(true);
    });
}
