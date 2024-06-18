use std::ops::{Deref, DerefMut};

use chrono::prelude::*;
pub use chrono::ParseError;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
#[serde(into = "String", try_from = "String")]
pub struct Date {
    inner: NaiveDate,
}
impl Date {
    const FORMAT: &'static str = "%d-%m-%Y";
    pub fn today() -> Self {
        Self {
            inner: Local::now().naive_local().date(),
        }
    }
    pub fn friendly_format(&self) -> String {
        self.inner.format("%d %B, %Y").to_string()
    }
}
impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.format(Self::FORMAT))
    }
}

impl From<Date> for String {
    fn from(val: Date) -> Self {
        val.to_string()
    }
}

impl TryFrom<&str> for Date {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        NaiveDate::parse_from_str(value, Self::FORMAT).map(|d| Self { inner: d })
    }
}

impl TryFrom<String> for Date {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Date as TryFrom<&str>>::try_from(&value)
    }
}

impl std::str::FromStr for Date {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Deref for Date {
    type Target = NaiveDate;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for Date {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<NaiveDate> for Date {
    fn from(value: NaiveDate) -> Self {
        Self { inner: value }
    }
}
