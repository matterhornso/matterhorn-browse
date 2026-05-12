// Matterhorn App — production binary entrypoint built on GPUI as a library.
//
// DIAGNOSTIC MODE: set MATTERHORN_RENDER_TEST=1 to mount a minimal text-only
// view instead of BrowserState. This isolates "does GPUI text render at all in
// our binary?" from "does it render through the BrowserState → OnboardingState
// entity chain?". Strip the diagnostic once we have an answer.

use std::borrow::Cow;

use gpui::{
    App, AppContext, Bounds, Context, Render, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use matterhorn_common::MatterhornConfig;
use matterhorn_viewport::BrowserState;

const FONT_BYTES: &[&[u8]] = &[
    include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"),
    include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-Italic.ttf"),
    include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-SemiBoldItalic.ttf"),
    include_bytes!("../../../assets/fonts/lilex/Lilex-Regular.ttf"),
    include_bytes!("../../../assets/fonts/lilex/Lilex-Bold.ttf"),
];

struct RenderTest;

impl Render for RenderTest {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
        // Four rows, each exercising a different fallback path. If any of these
        // render, that path works. If none render, GPUI text is broken in this
        // binary regardless of font / color settings.
        div()
            .size_full()
            .bg(rgb(0x0C0C0C))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .text_color(rgb(0xFFFFFF))
            .child(
                div()
                    .text_xl()
                    .font_family("IBM Plex Sans")
                    .child("ROW A — IBM Plex Sans, white on dark"),
            )
            .child(
                div()
                    .text_xl()
                    .font_family(".SystemUIFont")
                    .child("ROW B — .SystemUIFont, white on dark"),
            )
            .child(
                div()
                    .text_xl()
                    .child("ROW C — default font, inherited text_color"),
            )
            .child(
                div()
                    .text_xl()
                    .bg(rgb(0xFFFFFF))
                    .text_color(rgb(0x000000))
                    .px_3()
                    .py_1()
                    .child("ROW D — black on white box"),
            )
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let _guard = runtime.enter();

    let render_test = std::env::var("MATTERHORN_RENDER_TEST").ok().as_deref() == Some("1");

    let config = MatterhornConfig::load_or_default();
    if !render_test {
        if let Err(e) = config.save() {
            log::warn!("matterhorn: failed to persist config: {e}");
        }
    }

    application().run(move |cx: &mut App| {
        let fonts: Vec<Cow<'static, [u8]>> =
            FONT_BYTES.iter().map(|b| Cow::Borrowed(*b)).collect();
        if let Err(e) = cx.text_system().add_fonts(fonts) {
            log::error!("matterhorn: failed to register bundled fonts: {e:?}");
        } else {
            log::info!(
                "matterhorn: registered {} bundled font faces",
                FONT_BYTES.len()
            );
        }

        // Surface what GPUI sees as registered family names so we can spot
        // mismatches (e.g. font-kit reading "IBM Plex Sans Regular" instead of
        // the family name we expect).
        let families = cx.text_system().all_font_names();
        log::info!(
            "matterhorn: text_system sees {} font names; first 20: {:?}",
            families.len(),
            families.iter().take(20).collect::<Vec<_>>()
        );

        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        if render_test {
            log::info!("matterhorn: MATTERHORN_RENDER_TEST=1, mounting RenderTest");
            cx.open_window(opts, |_window, cx| cx.new(|_| RenderTest))
                .expect("failed to open window");
        } else {
            cx.open_window(opts, |_window, cx| {
                cx.new(|cx| BrowserState::new(cx, config.clone()))
            })
            .expect("failed to open window");
        }
        cx.activate(true);
    });
}
