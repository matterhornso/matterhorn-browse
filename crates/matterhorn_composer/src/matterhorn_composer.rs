use gpui::{
    div, prelude::*, rgb, App, FocusHandle, Focusable, InteractiveElement, KeyDownEvent,
    ParentElement, Render, Rgba, SharedString, Styled, Window,
};

const BG: u32 = 0x0C0C0C;
const SURFACE: u32 = 0x1C1C1E;
const SURFACE_ALT: u32 = 0x161618;
const BORDER: u32 = 0x2C2C2E;
const TEXT_MUTED: u32 = 0xA1A1A6;
const TEXT_DIM: u32 = 0x636366;
const TEXT: u32 = 0xFFFFFF;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Url,
    NaturalLanguage,
    Transaction,
    Unknown,
}

pub struct ComposerState {
    pub input_text: SharedString,
    pub mode: InputMode,
    pub history: Vec<SharedString>,
    pub suggestions_visible: bool,
    pub submitted: bool,
    pub focus_handle: FocusHandle,
    _saved_mode: Option<InputMode>,
}

impl ComposerState {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        Self {
            input_text: SharedString::from(""),
            mode: InputMode::Unknown,
            history: Vec::new(),
            suggestions_visible: false,
            submitted: false,
            focus_handle: cx.focus_handle(),
            _saved_mode: None,
        }
    }

    fn handle_char(&mut self, ch: char, cx: &mut gpui::Context<Self>) {
        if ch.is_control() {
            return;
        }
        let mut s = self.input_text.to_string();
        s.push(ch);
        self.input_text = SharedString::from(s);
        self.mode = detect_mode(&self.input_text);
        cx.notify();
    }

    fn handle_backspace(&mut self, cx: &mut gpui::Context<Self>) {
        let s = self.input_text.to_string();
        if s.is_empty() {
            return;
        }
        let mut chars: Vec<char> = s.chars().collect();
        chars.pop();
        let new_text: String = chars.into_iter().collect();
        self.input_text = SharedString::from(new_text);
        self.mode = detect_mode(&self.input_text);
        cx.notify();
    }

    fn handle_submit(&mut self, cx: &mut gpui::Context<Self>) {
        let text = self.input_text.clone();
        if text.is_empty() {
            return;
        }
        let saved_mode = self.mode.clone();
        if !self.history.contains(&text) {
            self.history.push(text.clone());
            if self.history.len() > 50 {
                self.history.remove(0);
            }
        }
        self.input_text = SharedString::from("");
        self.mode = InputMode::Unknown;
        self.suggestions_visible = false;
        self.submitted = true;
        self._saved_mode = Some(saved_mode);
        cx.notify();
    }

    pub fn take_submission(&mut self) -> Option<(SharedString, InputMode)> {
        if self.submitted {
            self.submitted = false;
            let last = self.history.last().cloned();
            let mode = self._saved_mode.take().unwrap_or(InputMode::Unknown);
            last.map(|text| (text, mode))
        } else {
            None
        }
    }

    fn mode_indicator(&self) -> (SharedString, Rgba) {
        match self.mode {
            InputMode::Url => (SharedString::from("URL"), rgb(SURFACE)),
            InputMode::NaturalLanguage => (SharedString::from("AI"), rgb(0x4A90D9)),
            InputMode::Transaction => (SharedString::from("TX"), rgb(0xE8A838)),
            InputMode::Unknown => (SharedString::from(""), rgb(SURFACE)),
        }
    }

    fn recent_suggestions(&self) -> Vec<&SharedString> {
        if self.input_text.is_empty() {
            self.history.iter().rev().take(6).collect()
        } else {
            self.history
                .iter()
                .rev()
                .filter(|h| h.contains(self.input_text.as_ref()))
                .take(6)
                .collect()
        }
    }
}

fn detect_mode(input: &str) -> InputMode {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return InputMode::Unknown;
    }
    let url_regex = regex_lite::Regex::new(
        r"(?i)^(https?://)?([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}([/?#].*)?$",
    )
    .ok();
    let tx_regex = regex_lite::Regex::new(
        r"(?i)\b(send|transfer|swap|bridge|stake)\b.*\b(ETH|SOL|USDC|USDT|MATIC|BTC)\b",
    )
    .ok();

    if let Some(ref re) = tx_regex {
        if re.is_match(trimmed) {
            return InputMode::Transaction;
        }
    }
    if let Some(ref re) = url_regex {
        if re.is_match(trimmed) {
            return InputMode::Url;
        }
    }
    InputMode::NaturalLanguage
}

impl Focusable for ComposerState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ComposerState {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let (mode_label, mode_color) = self.mode_indicator();
        let has_mode = !matches!(self.mode, InputMode::Unknown);
        let suggestions = if self.suggestions_visible {
            self.recent_suggestions()
        } else {
            Vec::new()
        };
        let display_text = if self.input_text.is_empty() {
            SharedString::from("Ask anything or enter a URL...")
        } else {
            self.input_text.clone()
        };

        div()
            .flex_col()
            .w_full()
            .on_key_down(cx.listener(
                |this, ev: &KeyDownEvent, _window, cx| match ev.keystroke.key.as_str() {
                    "backspace" => this.handle_backspace(cx),
                    "enter" | "return" => this.handle_submit(cx),
                    "space" => this.handle_char(' ', cx),
                    key if key.chars().count() == 1 => {
                        this.handle_char(key.chars().next().unwrap(), cx);
                    }
                    _ => {}
                },
            ))
            .child(
                div()
                    .flex_row()
                    .items_center()
                    .px_3()
                    .py_1()
                    .bg(rgb(SURFACE_ALT))
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .flex_1()
                            .flex_row()
                            .items_center()
                            .px_3()
                            .py_1p5()
                            .bg(rgb(BG))
                            .rounded_full()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .child(
                                div()
                                    .flex_row()
                                    .items_center()
                                    .px_2()
                                    .when(has_mode, move |el| {
                                        el.child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(mode_color)
                                                .text_xs()
                                                .text_color(rgb(TEXT))
                                                .child(mode_label.clone()),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px_2()
                                    .text_sm()
                                    .when(self.input_text.is_empty(), |el| {
                                        el.text_color(rgb(TEXT_DIM))
                                    })
                                    .when(!self.input_text.is_empty(), |el| {
                                        el.text_color(rgb(TEXT))
                                    })
                                    .child(display_text),
                            )
                            .child(
                                div()
                                    .flex_row()
                                    .items_center()
                                    .px_2()
                                    .text_xs()
                                    .text_color(rgb(TEXT_DIM))
                                    .child(SharedString::from("\u{2318}K")),
                            ),
                    ),
            )
            .when(!suggestions.is_empty(), |el| {
                el.child(
                    div()
                        .flex_col()
                        .bg(rgb(SURFACE))
                        .border_b_1()
                        .border_color(rgb(BORDER))
                        .children(suggestions.iter().map(|suggestion| {
                            div()
                                .px_4()
                                .py_1p5()
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .hover(|el| el.bg(rgb(SURFACE_ALT)))
                                .cursor_pointer()
                                .child((*suggestion).clone())
                                .into_any_element()
                        })),
                )
            })
    }
}
