use crate::date::Date;
use cocoon::Cocoon;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, fs::File, io};
use tui_textarea::TextArea;

#[derive(Debug, Clone, PartialEq)]
pub enum DiaryError {
    WrongPassword,
    InvalidFormat,
    OutOfRangeSize,
    NotFound,
    NotAccessible,
}
impl From<cocoon::Error> for DiaryError {
    fn from(value: cocoon::Error) -> Self {
        match value {
            cocoon::Error::Io(e) => e.into(),
            cocoon::Error::UnrecognizedFormat | cocoon::Error::TooShort => Self::InvalidFormat,
            cocoon::Error::Cryptography => Self::WrongPassword,
            cocoon::Error::TooLarge => Self::OutOfRangeSize,
        }
    }
}
impl From<io::Error> for DiaryError {
    fn from(value: io::Error) -> Self {
        if value.kind() == io::ErrorKind::NotFound {
            Self::NotFound
        } else {
            Self::NotAccessible
        }
    }
}
impl From<serde_json::Error> for DiaryError {
    fn from(value: serde_json::Error) -> Self {
        let _ = value;
        Self::InvalidFormat
    }
}
impl Display for DiaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidFormat => "Invalid File Format",
            Self::NotAccessible => "Cannot Access File",
            Self::NotFound => "File does not exist",
            Self::WrongPassword => "Wrong Password",
            Self::OutOfRangeSize => "File has invalid size",
        };
        write!(f, "{message}")
    }
}
impl std::error::Error for DiaryError {}
#[derive(Debug, Deserialize, Serialize)]
pub struct Diary {
    pub entries: HashMap<Date, String>,
}
impl Default for Diary {
    fn default() -> Self {
        Self::new()
    }
}
impl Diary {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
    pub fn read_jrnl(path: &str, password: &str) -> Result<Self, DiaryError> {
        let cocoon = Cocoon::new(password.as_bytes());
        let bytes = cocoon.parse(&mut File::open(path)?)?;
        let string = String::from_utf8_lossy(bytes.as_slice()).into_owned();
        Ok(serde_json::from_str(&string)?)
    }
    pub fn write_to(&self, path: &str, password: &str) -> Result<(), DiaryError> {
        let mut cocoon = Cocoon::new(password.as_bytes());
        cocoon.dump(
            serde_json::to_string(self)?.into_bytes(),
            &mut File::create(path)?,
        )?;
        Ok(())
    }
}
impl From<&HashMap<Date, TextArea<'_>>> for Diary {
    fn from(value: &HashMap<Date, TextArea<'_>>) -> Self {
        Self {
            entries: value
                .iter()
                .map(|(k, v)| {
                    (
                        *k,
                        v.lines()
                            .iter()
                            .flat_map(|k| -> [&str; 2] { [k, "\n"] })
                            .collect::<String>(),
                    )
                })
                .collect(),
        }
    }
}
impl<'a> From<Diary> for HashMap<Date, TextArea<'a>> {
    fn from(val: Diary) -> Self {
        val.entries
            .into_iter()
            .map(|(k, v)| (k, TextArea::from(v.split('\n'))))
            .collect()
    }
}
