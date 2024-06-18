pub(crate) use tui_textarea::{Input, Key, TextArea};

pub fn input(ta: &mut TextArea, input: impl Into<Input>) -> bool {
    match input.into() {
        Input {
            key: Key::Char('v'),
            ctrl: true,
            ..
        }
        | Input {
            key: Key::Paste, ..
        } => ta.paste(),
        Input {
            key: Key::Char('z'),
            ctrl: true,
            ..
        } => ta.undo(),
        Input {
            key: Key::Char('y'),
            ctrl: true,
            ..
        } => ta.redo(),
        Input {
            key: Key::Char('a'),
            ctrl: true,
            ..
        } => {
            ta.select_all();
            false
        }
        i => ta.input(i),
    }
}
