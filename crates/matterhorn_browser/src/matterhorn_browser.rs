// Matterhorn Browser — GPUI-based Web3 Browser

use assets::Assets;
use gpui::{
    Application, AppContext, Bounds, Point, WindowBounds, WindowOptions, px, size,
};
use matterhorn_common::MatterhornConfig;
use matterhorn_viewport::BrowserState;

fn main() {
    let platform = gpui_platform::current_platform(false);
    let app = Application::with_platform(platform).with_assets(Assets);

    app.run(move |cx| {
        // Bake the embedded font set (IBM Plex Sans, Lilex, etc.) into the
        // text system. Without this, every text element renders with width 0
        // and the UI looks blank against the dark background.
        if let Err(e) = Assets.load_fonts(cx) {
            eprintln!("matterhorn: failed to load fonts: {e}");
        }

        let config = MatterhornConfig::load_or_default();
        // Persist the resolved config so first-launch users get a populated
        // file they can edit. Failures are non-fatal.
        if let Err(e) = config.save() {
            eprintln!("matterhorn: failed to persist config: {e}");
        }

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(800.0)),
            })),
            ..Default::default()
        };

        let _window = cx
            .open_window(options, |_window, cx| {
                cx.new(|cx| BrowserState::new(cx, config))
            })
            .unwrap();

        cx.activate(true);
    });
}
