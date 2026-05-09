// Matterhorn Browser — GPUI-based Web3 Browser

use gpui::{Application, AppContext, WindowBounds, WindowOptions, Bounds, Point, SharedString, size, px};
use matterhorn_viewport::BrowserState;
use matterhorn_common::MatterhornConfig;

struct Assets;

impl gpui::AssetSource for Assets {
    fn load(&self, _path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(None)
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

fn main() {
    let platform = gpui_platform::current_platform(false);
    let app = Application::with_platform(platform).with_assets(Assets);

    app.run(move |cx| {
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

        let window = cx.open_window(options, |_window, cx| {
            cx.new(|cx| BrowserState::new(cx, config))
        })
        .unwrap();

        cx.activate(true);
    });
}
