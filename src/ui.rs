pub use std::collections::HashMap;

pub use chrono::{Days, Months};
pub use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListState},
};
use text_box::TextBox;
pub use tui_textarea::{Input, Key, TextArea};

pub use crate::{clear, date::Date};
pub mod text_box {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        text::Text,
        widgets::{Block, Widget},
    };

    pub struct TextBox<'a> {
        text: Text<'a>,
        block: Block<'a>,
    }
    impl<'a> TextBox<'a> {
        pub fn new<T: Into<Text<'a>>>(text: T, block: Block<'a>) -> Self {
            Self {
                text: text.into().centered(),
                block,
            }
        }
        pub fn get_block(&self) -> Block<'a> {
            self.block.clone()
        }
        pub fn set_block(&mut self, block: Block<'a>) {
            self.block = block
        }
    }
    impl<'a> Widget for TextBox<'a> {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            let ta = self.block.inner(area);
            self.block.render(area, buf);
            self.text.render(ta, buf);
        }
    }
    impl<'a, T> From<T> for TextBox<'a>
    where
        T: Into<Text<'a>>,
    {
        fn from(value: T) -> Self {
            Self {
                text: value.into(),
                block: Block::bordered(),
            }
        }
    }
}
pub(crate) mod editor {
    use super::*;
    pub fn editor_ui<T>(buf: &mut Buffer, ta: &TextArea, entries: &HashMap<Date, T>, date: &Date) {
        let areas = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(65), Constraint::Min(20)],
        )
        .split(buf.area);
        ta.widget().render(areas[0], buf);
        sidebar(areas[1], buf, entries, date);
    }
    fn sidebar<T>(area: Rect, buf: &mut Buffer, entries: &HashMap<Date, T>, date: &Date) {
        let areas = Layout::new(
            Direction::Vertical,
            [Constraint::Min(20), Constraint::Percentage(70)],
        )
        .split(area);
        let tb = Block::default().borders(Borders::all());
        let shortcuts = List::new([
            "Quit  :  <Esc>",
            "Save  :  <Ctrl+S>",
            "Date  :  <Alt+D>",
            "Delete:  <Ctrl+Delete>",
        ])
        .block(tb);
        <List as Widget>::render(shortcuts, areas[1], buf);
        let mut entries = entries
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        entries.sort();
        let to_select = entries.binary_search(&date.to_string()).ok();
        let tb = Block::new()
            .borders(Borders::all())
            .title_top(" Dates you've journaled for ")
            .bold();
        let el = List::new(entries)
            .block(tb)
            .highlight_style(Style::new().bg(Color::Gray).fg(Color::White));
        let mut els = ListState::default().with_selected(to_select);
        <List as StatefulWidget>::render(el, area, buf, &mut els);
    }

    pub fn pre_exit_ui(buf: &mut Buffer) {
        let area = buf.area;
        let tb = TextBox::new(
            Text::from("Do you want to quit without saving? (y\\n)").bold(),
            Block::bordered(),
        );
        tb.render(area, buf)
    }
}

pub(crate) mod date_selection {
    use text_box::TextBox;

    use super::*;
    enum CurrentlySelected {
        Date,
        Month,
        Year,
    }
    impl CurrentlySelected {
        pub fn next(&self) -> Self {
            match self {
                CurrentlySelected::Date => CurrentlySelected::Month,
                CurrentlySelected::Month => CurrentlySelected::Year,
                CurrentlySelected::Year => CurrentlySelected::Date,
            }
        }
        pub fn prev(&self) -> Self {
            match self {
                CurrentlySelected::Date => CurrentlySelected::Year,
                CurrentlySelected::Month => CurrentlySelected::Date,
                CurrentlySelected::Year => CurrentlySelected::Month,
            }
        }
    }
    pub(crate) struct DateSelection {
        date: Date,
        selection: CurrentlySelected,
    }
    impl DateSelection {
        pub fn new(date: Date) -> Self {
            Self {
                date,
                selection: CurrentlySelected::Date,
            }
        }
        pub fn increment_selected(&mut self) {
            match self.selection {
                CurrentlySelected::Date => {
                    self.date = self
                        .date
                        .checked_add_days(Days::new(1))
                        .unwrap_or(*self.date)
                        .into()
                }
                CurrentlySelected::Month => {
                    self.date = self
                        .date
                        .checked_add_months(Months::new(1))
                        .unwrap_or(*self.date)
                        .into()
                }
                CurrentlySelected::Year => {
                    self.date = self
                        .date
                        .checked_add_months(Months::new(12))
                        .unwrap_or(*self.date)
                        .into()
                }
            }
        }
        pub fn decrement_selected(&mut self) {
            match self.selection {
                CurrentlySelected::Date => {
                    self.date = self
                        .date
                        .checked_sub_days(Days::new(1))
                        .unwrap_or(*self.date)
                        .into()
                }
                CurrentlySelected::Month => {
                    self.date = self
                        .date
                        .checked_sub_months(Months::new(1))
                        .unwrap_or(*self.date)
                        .into()
                }
                CurrentlySelected::Year => {
                    self.date = self
                        .date
                        .checked_sub_months(Months::new(12))
                        .unwrap_or(*self.date)
                        .into()
                }
            }
        }
        pub fn select_next(&mut self) {
            self.selection = self.selection.next();
        }
        pub fn select_prev(&mut self) {
            self.selection = self.selection.prev();
        }
        pub fn date(&self) -> Date {
            self.date
        }
    }
    fn selected_block<'a, T: Into<Line<'a>>>(title: T) -> Block<'a> {
        Block::bordered()
            .bold()
            .white()
            .title_bottom(title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Thick)
    }
    impl Widget for &DateSelection {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            let areas = Layout::new(
                Direction::Horizontal,
                vec![
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                ],
            )
            .split(area);
            let default_block = Block::bordered().border_style(Style::new().fg(Color::DarkGray));
            let date_w = TextBox::new(
                self.date.format("%d").to_string(),
                if let CurrentlySelected::Date = self.selection {
                    selected_block("Date")
                } else {
                    default_block.clone()
                },
            );
            let month_w = TextBox::new(
                self.date.format("%B").to_string(),
                if let CurrentlySelected::Month = self.selection {
                    selected_block("Month")
                } else {
                    default_block.clone()
                },
            );
            let year_w = TextBox::new(
                self.date.format("%Y").to_string(),
                if let CurrentlySelected::Year = self.selection {
                    selected_block("Year")
                } else {
                    default_block.clone()
                },
            );
            date_w.render(areas[0], buf);
            month_w.render(areas[1], buf);
            year_w.render(areas[2], buf);
        }
    }
    pub(crate) fn get_date_ui(buf: &mut Buffer, elements: &mut DateSelection) {
        let border = Block::bordered()
            .title_top("Choose Date")
            .title_bottom("[ <Space>/+: Increase value | '-' : Decrease Value | <Enter>: Submit ]")
            .title_alignment(Alignment::Center)
            .bold();
        let area = border.inner(buf.area);
        border.render(buf.area, buf);
        let area = centered(area, Constraint::Percentage(10), Constraint::Percentage(40));
        elements.render(area, buf);
    }
}

pub(crate) mod password_form {
    use super::*;
    enum PasswordFormMode {
        Typing,
        Retyping,
        Submitted,
    }
    pub struct PasswordForm<'a> {
        og: TextArea<'a>,
        retype: TextArea<'a>,
        mode: PasswordFormMode,
        matching: bool,
    }
    impl<'a> Default for PasswordForm<'a> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<'a> PasswordForm<'a> {
        fn inactive_block(block: Block<'a>) -> Block<'a> {
            block.style(Style::new().fg(Color::DarkGray).bg(Color::Black))
        }
        fn active_block(block: Block<'a>) -> Block<'a> {
            block.style(Style::new().fg(Color::Gray).bg(Color::DarkGray))
        }
        pub fn new() -> Self {
            let mut i = Self {
                og: password_ta(" Enter Password: "),
                retype: password_ta(" Retype password: "),
                mode: PasswordFormMode::Typing,
                matching: true,
            };
            i.typing();
            i
        }
        pub fn input(&mut self, input: impl Into<Input>) -> Option<String> {
            let enter_key = Input {
                key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            };
            let input = input.into();
            match self.mode {
                PasswordFormMode::Typing => {
                    if enter_key == input {
                        self.og
                            .set_block(Self::inactive_block(self.og.block().unwrap().clone()));
                        self.retype
                            .set_block(Self::active_block(self.retype.block().unwrap().clone()));
                        self.mode = PasswordFormMode::Retyping;
                    } else {
                        self.og.input(input);
                    }
                    None
                }
                PasswordFormMode::Retyping => {
                    if enter_key == input {
                        self.mode = PasswordFormMode::Submitted;
                    } else {
                        self.retype.input(input);
                    }
                    None
                }
                PasswordFormMode::Submitted => {
                    if self
                        .og
                        .lines()
                        .first()
                        .map(|x| x.as_str())
                        .unwrap_or_default()
                        != self
                            .retype
                            .lines()
                            .first()
                            .map(|f| f.as_str())
                            .unwrap_or_default()
                    {
                        self.matching = false;
                        clear(&mut self.og);
                        clear(&mut self.retype);
                        self.typing();
                        None
                    } else {
                        Some(self.og.lines()[0].clone())
                    }
                }
            }
        }
        pub fn retyping(&mut self) {
            self.og
                .set_block(Self::inactive_block(self.og.block().unwrap().clone()));
            self.retype
                .set_block(Self::active_block(self.retype.block().unwrap().clone()));
            self.mode = PasswordFormMode::Retyping;
        }
        pub fn typing(&mut self) {
            self.og
                .set_block(Self::active_block(self.og.block().unwrap().clone()));
            self.retype
                .set_block(Self::inactive_block(self.retype.block().unwrap().clone()));
            self.mode = PasswordFormMode::Typing;
        }
    }

    impl Widget for &PasswordForm<'_> {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            let areas = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Percentage(40),
                    Constraint::Max(1),
                    Constraint::Percentage(40),
                    Constraint::Min(1),
                ],
            )
            .split(area);
            self.og.widget().render(areas[0], buf);
            self.retype.widget().render(areas[2], buf);
            if !self.matching {
                Line::raw("Passwords don't match")
                    .centered()
                    .render(areas[3], buf);
            }
        }
    }
    pub fn password_form_ui(pf: &PasswordForm, buf: &mut Buffer) {
        let area = centered(
            buf.area,
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        );
        pf.render(area, buf);
    }
    fn password_ta<'a>(title: impl Into<Line<'a>>) -> TextArea<'a> {
        let mut ta = TextArea::default();
        ta.set_block(Block::bordered().title_top(title));
        ta.set_mask_char('*');
        ta.set_cursor_line_style(Style::default().fg(ratatui::style::Color::Gray));
        ta
    }
}

pub fn delete_ui(date: Date, buf: &mut Buffer) {
    TextBox::from(format!(
        "Do you want to delete the entry for {}? (y\\n)",
        date.friendly_format()
    ))
    .render(buf.area, buf);
}
pub fn create_file(area: Rect, buf: &mut Buffer, path: &str) {
    TextBox::from(format!("Do you want to create \"{path}\" ? (y/n)")).render(area, buf)
}

pub fn centered_input_box(ta: &TextArea<'_>, buf: &mut Buffer) {
    let area = centered(
        buf.area,
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    );
    ta.widget().render(area, buf);
}

pub fn centered(area: Rect, horizontal_margin: Constraint, vertical_margin: Constraint) -> Rect {
    Layout::new(
        Direction::Vertical,
        [vertical_margin, Constraint::Min(1), vertical_margin],
    )
    .split(
        Layout::new(
            Direction::Horizontal,
            [horizontal_margin, Constraint::Min(1), horizontal_margin],
        )
        .split(area)[1],
    )[1]
}
