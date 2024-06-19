use std::collections::HashMap;

use crate::{
    app::{App, AppMode},
    diary::{Diary, DiaryError},
    ui::Date,
};
#[derive(Debug, clap::Parser)]
pub struct Arguments {
    #[arg(short, long)]
    file: Option<String>,
    #[arg(short, long, requires("file"))]
    password: Option<String>,
    #[arg(short,long,value_name("DATE: DD-MM-YYYY"))]
    date: Option<Date>
}
impl TryFrom<Arguments> for App<'_> {
    type Error = DiaryError;
    fn try_from(value: Arguments) -> Result<Self, Self::Error> {
        let mut app = App::new()?;
        if let Some(d) = value.date {
            app.date = d
        }
        match value {
            Arguments {
                file: Some(file),
                password: None,
                ..
            } => match Diary::read_jrnl(&file, "") {
                Ok(entries) => {
                    app.path = file;
                    app.password = String::new();
                    app.mode = AppMode::Edit;
                    app.entries = HashMap::from(entries);
                    app.entries
                        .entry(Date::today())
                        .or_insert(App::input_area(Date::today(), None));
                    app.entries
                        .iter_mut()
                        .for_each(|(date, input)| App::setup_input_area(input, *date));
                }
                Err(DiaryError::WrongPassword) => {
                    app.path = file;
                    app.mode = AppMode::Password;
                }
                Err(e) => return Err(e),
            },
            Arguments {
                file: Some(file),
                password: Some(password),
                ..
            } => {
                app.password = password;
                app.path = file;
                app.try_load()?;
                app.mode = AppMode::Edit;
            }
            _ => (),
        }
        Ok(app)
    }
}
