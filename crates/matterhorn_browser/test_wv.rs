use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Wry Test")
        .with_inner_size(tao::dpi::LogicalSize::new(600.0, 400.0))
        .build(&event_loop)
        .unwrap();

    let _webview = WebViewBuilder::new()
        .with_html("<html><body style='background:#111;color:#fff;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0'><h1>If you can read this, Wry works ✅</h1></body></html>")
        .with_devtools(true)
        .build(&window)
        .unwrap();

    dbg!("WebView built successfully");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            *control_flow = ControlFlow::Exit;
        }
    });
}
