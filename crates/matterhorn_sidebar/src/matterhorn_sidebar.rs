use gpui::{
    IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px, rgb,
};

const ACCENT: u32 = 0xD1F2FF;
const SURFACE: u32 = 0x1C1C1E;
const BORDER: u32 = 0x2C2C2E;
const TEXT_MUTED: u32 = 0xA1A1A6;
const TEXT: u32 = 0xFFFFFF;

pub struct HistoryAction {
    pub description: SharedString,
    pub timestamp: SharedString,
}

pub struct SidebarState {
    pub visible: bool,
    pub recent_actions: Vec<HistoryAction>,
    pub context_summary: SharedString,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            visible: false,
            recent_actions: Vec::new(),
            context_summary: SharedString::from("No active browsing context."),
        }
    }

    pub fn toggle(&mut self, cx: &mut gpui::Context<Self>) {
        self.visible = !self.visible;
        cx.notify();
    }

    pub fn add_action(&mut self, description: SharedString, cx: &mut gpui::Context<Self>) {
        self.recent_actions.insert(
            0,
            HistoryAction {
                description,
                timestamp: SharedString::from("just now"),
            },
        );
        if self.recent_actions.len() > 20 {
            self.recent_actions.truncate(20);
        }
        cx.notify();
    }

    pub fn set_context(&mut self, summary: SharedString, cx: &mut gpui::Context<Self>) {
        self.context_summary = summary;
        cx.notify();
    }
}

impl Render for SidebarState {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div().into_any_element();
        }

        div()
            .flex_col()
            .w(px(280.0))
            .h_full()
            .bg(rgb(SURFACE))
            .border_l_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(ACCENT))
                            .child("AI Context"),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_MUTED))
                            .child("CURRENT CONTEXT"),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_sm()
                            .text_color(rgb(TEXT))
                            .child(self.context_summary.clone()),
                    ),
            )
            .child(
                div()
                    .flex_col()
                    .flex_1()
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_MUTED))
                            .child("RECENT ACTIONS"),
                    )
                    .children(self.recent_actions.iter().take(10).map(|action| {
                        div()
                            .flex_col()
                            .mt_1()
                            .py_1()
                            .border_b_1()
                            .border_color(rgb(BORDER))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(TEXT))
                                    .child(action.description.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(action.timestamp.clone()),
                            )
                            .into_any_element()
                    })),
            )
            .into_any_element()
    }
}
