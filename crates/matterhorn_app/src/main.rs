// Matterhorn App — production binary entrypoint built on GPUI as a library.
//
// This is the Option B architecture from REVIEW.md. The legacy
// matterhorn_browser crate wired through Zed's heavyweight Application::with_*
// builder which depended on init scaffolding (settings, themes, asset source)
// we never wired, producing a blank UI. This entry mirrors the
// crates/gpui/examples/* pattern: application().run(...), one open_window,
// platform-native fonts, no Zed-specific init.
//
// The tokio runtime is entered before app.run so the existing async paths in
// matterhorn_viewport (Handle::current() in start_balance_fetches /
// confirm_transaction) keep working without depending on the Zed-internal
// gpui_tokio bridge.

use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;
use matterhorn_common::MatterhornConfig;
use matterhorn_viewport::BrowserState;

fn main() {
    // Multi-threaded tokio runtime — kept alive for the lifetime of the app
    // so async work scheduled via cx.spawn + Handle::current() always has a
    // runtime to attach to.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let _guard = runtime.enter();

    let config = MatterhornConfig::load_or_default();
    if let Err(e) = config.save() {
        eprintln!("matterhorn: failed to persist config: {e}");
    }

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| BrowserState::new(cx, config.clone())),
        )
        .expect("failed to open window");
        cx.activate(true);
    });
}
