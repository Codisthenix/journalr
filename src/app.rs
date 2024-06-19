use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, stderr, Stderr},
    time::Duration,
};

use crossterm::{
    event::{self, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Style, Stylize},
    widgets::Block,
    Frame, Terminal,
};
use tui_textarea::TextArea;

use crate::{
    clear,
    date::Date,
    diary::{Diary, DiaryError},
    ui::{
        centered_input_box, create_file,
        date_selection::{get_date_ui, DateSelection},
        delete_ui,
        editor::{editor_ui, pre_exit_ui},
        password_form::{password_form_ui, PasswordForm},
    },
};

pub enum AppMode {
    Edit,
    AskToSave,
    Password,
    Exit,
    SetDate,
    Delete,
    GetFile,
}
impl Default for AppMode {
    fn default() -> Self {
        Self::Edit
    }
}
pub struct App<'a> {
    pub path: String,
    pub date: Date,
    pub(crate) entries: HashMap<Date, TextArea<'a>>,
    terminal: Terminal<CrosstermBackend<Stderr>>,
    pub(crate) mode: AppMode,
    saved: bool,
    pub password: String,
}
impl<'a> App<'a> {
    pub(crate) fn setup_input_area(input: &mut TextArea<'_>, date: Date) {
        let block = Block::bordered()
            .title_top(format!(" Diary entry: {} ", date.friendly_format()))
            .border_style(Style::new().bold().white());
        input.set_block(block);
        input.set_cursor_line_style(Style::default().fg(ratatui::style::Color::Gray));
        input.set_line_number_style(Style::default().bg(ratatui::style::Color::DarkGray));
    }
    pub(crate) fn input_area(date: Date, cont: Option<&String>) -> TextArea<'a> {
        let mut input = match cont {
            None => TextArea::default(),
            Some(s) => TextArea::from(s.split('\n')),
        };
        Self::setup_input_area(&mut input, date);
        input
    }
    pub fn new() -> io::Result<Self> {
        let date = Date::today();
        let entries = HashMap::new();
        Ok(App {
            path: String::new(),
            entries,
            date,
            terminal: Terminal::new(CrosstermBackend::new(stderr()))?,
            mode: AppMode::GetFile,
            saved: true,
            password: String::new(),
        })
    }
    pub fn save(&mut self) {
        Diary::from(&self.entries)
            .write_to(&self.path, &self.password)
            .unwrap_or_else(|e| println!("Error: {e}"));
        println!("Diary saved");
        self.saved = true;
    }

    fn create_file(&mut self, path: &str) -> io::Result<bool> {
        loop {
            self.terminal.draw(|f| {
                let buf = f.buffer_mut();
                let area = buf.area;
                create_file(area, buf, path)
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Ok(Event::Key(k)) = read() {
                    if KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE) == k {
                        break Ok(true);
                    } else if KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE) == k {
                        break Ok(false);
                    }
                }
            }
        }
    }
    fn new_password(&mut self) -> io::Result<()> {
        let mut w = PasswordForm::new();
        loop {
            self.terminal.draw(|f| {
                let buf = f.buffer_mut();
                password_form_ui(&w, buf)
            })?;
            if let Ok(Event::Key(k)) = read() {
                if let Some(pwd) = w.input(k) {
                    self.password = pwd;
                    break;
                }
            }
        }
        Ok(())
    }
    /// Asks the user if new file 'path' is to be created and set password as well as default contents of the file.
    /// Returns true if user chose to create file
    fn new_file(&mut self, path: &str) -> io::Result<bool> {
        if self.create_file(path)? {
            File::create(path)?;
            self.path = path.to_owned();
            self.new_password()?;
            self.entries = HashMap::from([(self.date, Self::input_area(self.date, None))]);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn edit_view(&mut self) -> io::Result<()> {
        loop {
            self.terminal.draw(|f: &mut Frame| {
                let buf = f.buffer_mut();
                editor_ui(buf, &self.entries[&self.date], &self.entries, &self.date)
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Ok(Event::Key(k)) = read() {
                    if KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) == k {
                        self.mode = AppMode::AskToSave;
                        break;
                    } else if KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL) == k {
                        self.save();
                    } else if KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT) == k {
                        self.mode = AppMode::SetDate;
                        break;
                    } else if KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL) == k {
                        self.mode = AppMode::Delete;
                        break;
                    } else if self.entries.get_mut(&self.date).unwrap().input(k) {
                        self.saved = false;
                    }
                }
            }
        }
        Ok(())
    }
    /// Unlock the file `self.path` using `self.password` and read it into `self.entries`.
    ///
    /// Returns error if:
    /// - Password is wrong
    /// - File cannot be accesed
    pub(crate) fn try_load(&mut self) -> Result<(), DiaryError> {
        self.entries = HashMap::from(Diary::read_jrnl(&self.path, &self.password)?);
        self.entries
            .entry(self.date)
            .or_insert(Self::input_area(self.date, None));
        self.entries
            .iter_mut()
            .for_each(|(date, input)| Self::setup_input_area(input, *date));
        Ok(())
    }

    fn get_password(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut ta = TextArea::default();
        ta.set_block(Block::bordered().title_top("Enter password"));
        ta.set_mask_char('*');
        ta.set_cursor_line_style(Style::default().fg(ratatui::style::Color::Gray));
        loop {
            self.terminal.draw(|f| {
                centered_input_box(&ta, f.buffer_mut());
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Ok(Event::Key(k)) = read() {
                    if KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) == k {
                        self.mode = AppMode::Exit;
                        break;
                    } else if KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE) == k {
                        let password = ta.lines().first().map(|x| x.as_str());
                        if let Some(s) = password {
                            self.password = s.to_string();
                            if self.try_load().is_ok() {
                                self.mode = AppMode::Edit;
                                break;
                            } else {
                                clear(&mut ta);
                                ta.set_placeholder_text("Wrong Password");
                                continue;
                            }
                        }
                    }
                    ta.input(k);
                }
            }
        }
        Ok(())
    }

    fn pre_exit(&mut self) -> io::Result<()> {
        if !self.saved {
            loop {
                self.terminal.draw(|f: &mut Frame| {
                    let buf = f.buffer_mut();
                    pre_exit_ui(buf);
                })?;
                if event::poll(std::time::Duration::from_millis(16))? {
                    if let Ok(Event::Key(k)) = read() {
                        if KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) == k
                            || KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE) == k
                        {
                            self.mode = AppMode::Edit;
                            break;
                        } else if KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE) == k {
                            self.mode = AppMode::Exit;
                            break;
                        }
                    }
                }
            }
        } else {
            self.mode = AppMode::Exit;
        }
        Ok(())
    }
    fn set_date_ui(&mut self) -> io::Result<Option<Date>> {
        let mut uis = DateSelection::new(self.date);
        loop {
            self.terminal.draw(|f| {
                get_date_ui(f.buffer_mut(), &mut uis);
            })?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Ok(Event::Key(k)) = read() {
                    if KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE) == k
                        || KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE) == k
                        || KeyEvent::new(KeyCode::Up, KeyModifiers::NONE) == k
                    {
                        uis.increment_selected();
                    } else if KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE) == k
                        || KeyEvent::new(KeyCode::Down, KeyModifiers::NONE) == k
                    {
                        uis.decrement_selected();
                    } else if KeyEvent::new(KeyCode::Right, KeyModifiers::NONE) == k {
                        uis.select_next();
                    } else if KeyEvent::new(KeyCode::Left, KeyModifiers::NONE) == k {
                        uis.select_prev();
                    } else if KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE) == k {
                        self.mode = AppMode::Edit;
                        return Ok(Some(uis.date()));
                    } else if KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) == k {
                        self.mode = AppMode::Edit;
                        return Ok(None);
                    }
                }
            }
        }
    }
    fn delete(&mut self) -> io::Result<()> {
        loop {
            self.terminal
                .draw(|f| delete_ui(self.date, f.buffer_mut()))?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Ok(Event::Key(k)) = read() {
                    if KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE) == k
                        || KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE) == k
                    {
                        self.mode = AppMode::Edit;
                        break;
                    } else if KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE) == k {
                        self.entries.remove(&self.date);
                        self.mode = AppMode::SetDate;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    fn get_input(
        &mut self,
        title: &str,
        placeholder: &str,
        mask: Option<char>,
    ) -> Result<Option<String>, io::Error> {
        let mut input_area = TextArea::default();
        input_area.set_block(Block::bordered().title_top(title));
        input_area.set_cursor_style(Style::new().bg(ratatui::style::Color::White));
        input_area.set_placeholder_text(placeholder);
        if let Some(c) = mask {
            input_area.set_mask_char(c);
        }
        loop {
            self.terminal
                .draw(|f| centered_input_box(&input_area, f.buffer_mut()))?;
            if event::poll(Duration::from_millis(20))? {
                if let Ok(event) = read() {
                    if let Event::Key(key) = event {
                        if KeyEventKind::Release != key.kind {
                            match key.code {
                                KeyCode::Enter => {
                                    return Ok(Some(
                                        input_area
                                            .lines()
                                            .first()
                                            .map(|f| f.to_owned())
                                            .unwrap_or_default(),
                                    ))
                                }
                                KeyCode::Esc => return Ok(None),
                                _ => (),
                            };
                        }
                    }
                    input_area.input(event);
                }
            }
        }
    }
    fn open_file_rw(&mut self) -> Result<(), Box<dyn Error>> {
        let mut ph = "";
        loop {
            let filename = match self.get_input("Enter name of File to open", ph, None)? {
                Some(file) => file,
                None => {
                    self.mode = AppMode::Exit;
                    break;
                }
            };
            match Diary::read_jrnl(&filename, "") {
                Ok(entries) => {
                    self.mode = AppMode::Edit;
                    self.entries = HashMap::from(entries);
                    self.entries
                        .entry(Date::today())
                        .or_insert(Self::input_area(Date::today(), None));
                    self.entries
                        .iter_mut()
                        .for_each(|(date, input)| Self::setup_input_area(input, *date));
                    break;
                }
                Err(e) => match e {
                    DiaryError::WrongPassword => {
                        self.path = filename;
                        self.mode = AppMode::Password;
                        break;
                    }
                    DiaryError::InvalidFormat => {
                        ph = "Invalid File Format";
                    }
                    DiaryError::OutOfRangeSize => {
                        ph = "File too Large";
                    }
                    DiaryError::NotFound => match self.new_file(&filename) {
                        Ok(created) => {
                            if created {
                                self.path = filename;
                                self.mode = AppMode::Edit;
                                break;
                            } else {
                                self.mode = AppMode::Exit;
                                break;
                            }
                        }
                        Err(_) => {
                            ph = "File could not be created";
                        }
                    },
                    DiaryError::NotAccessible => {
                        ph = "File Cannot be Accesed";
                    }
                },
            };
        }
        Ok(())
    }
    pub fn exit(self) -> io::Result<()> {
        disable_raw_mode()?;
        stderr().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        stderr().execute(EnterAlternateScreen)?;
        self.entries.entry(self.date).or_insert(Self::input_area(self.date, None));
        loop {
            match self.mode {
                AppMode::Edit => self.edit_view()?,
                AppMode::Password => self.get_password()?,
                AppMode::AskToSave => self.pre_exit()?,
                AppMode::SetDate => {
                    if let Some(date) = self.set_date_ui()? {
                        self.entries
                            .entry(date)
                            .or_insert_with(|| Self::input_area(date, None));
                        self.date = date;
                    }
                }
                AppMode::Exit => return Ok(self.exit()?),
                AppMode::Delete => self.delete()?,
                AppMode::GetFile => self.open_file_rw()?,
            }
        }
    }
}

#[test]
fn test() {
    let app = App::new().unwrap();
    // app.mode = AppMode::NewFile;
    app.run().unwrap()
}
