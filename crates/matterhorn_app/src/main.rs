// Matterhorn App — production binary entrypoint built on GPUI as a library.
//
// The tokio runtime is entered before app.run so the existing async paths in
// matterhorn_viewport (Handle::current() in start_balance_fetches /
// confirm_transaction) keep working without depending on the Zed-internal
// gpui_tokio bridge.
//
// Fonts are baked into the binary and registered with the text system before
// the window opens. Without this, the default font lookup resolves to
// `.SystemUIFont` → `.AppleSystemUIFont`, which font-kit's SystemSource can
// fail to load on some macOS configurations — leaving every text node empty
// while backgrounds and borders still paint. We bake IBM Plex Sans (UI) and
// Lilex (mono) so the binary is self-sufficient.

use std::borrow::Cow;

use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
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

fn main() {
    // Initialize logging so GPUI's font-loading warnings, panics in glyph
    // rasterization, and other diagnostics are surfaced on stderr. Default to
    // `info` so first-run users see something useful without setting RUST_LOG.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let _guard = runtime.enter();

    let config = MatterhornConfig::load_or_default();
    if let Err(e) = config.save() {
        log::warn!("matterhorn: failed to persist config: {e}");
    }

    application().run(move |cx: &mut App| {
        let fonts: Vec<Cow<'static, [u8]>> =
            FONT_BYTES.iter().map(|b| Cow::Borrowed(*b)).collect();
        if let Err(e) = cx.text_system().add_fonts(fonts) {
            log::error!("matterhorn: failed to register bundled fonts: {e:?}");
        } else {
            log::info!("matterhorn: registered {} bundled font faces", FONT_BYTES.len());
        }

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
