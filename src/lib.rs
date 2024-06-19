use diary::{Diary, DiaryFromFileError};
use tui_textarea::TextArea;

pub mod app;
pub mod args;
pub mod date;
pub mod diary;
pub mod keybindings;
pub mod ui;
pub fn clear(ta: &mut TextArea<'_>) {
    ta.move_cursor(tui_textarea::CursorMove::Jump(0, 0));
    ta.delete_str(ta.lines().iter().fold(0, |len, x| len + 1 + x.len()));
}
pub fn check_format(path: &str) -> bool {
    let res = Diary::read_jrnl(path, "");
    res.is_ok() || res.is_err_and(|e| !(e == DiaryFromFileError::WrongPassword))
}
