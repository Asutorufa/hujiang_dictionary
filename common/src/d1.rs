use std::fmt;

use serde::{Deserialize, Serialize};
use value2struct::FromValueVec;

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Word {
    pub word: String,
    pub explain: String,
    add_time: i64,
    update_time: i64,
    reminder_time: i64,
}

#[derive(Debug)]
pub struct D1Error(pub String);

unsafe impl Send for D1Error {}

impl fmt::Display for D1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for D1Error {}

impl From<&str> for D1Error {
    fn from(s: &str) -> Self {
        D1Error(s.to_string())
    }
}

impl From<reqwest::Error> for D1Error {
    fn from(value: reqwest::Error) -> Self {
        D1Error(value.to_string())
    }
}

impl From<serde_json::Error> for D1Error {
    fn from(value: serde_json::Error) -> Self {
        D1Error(value.to_string())
    }
}

impl From<String> for D1Error {
    fn from(s: String) -> Self {
        D1Error(s)
    }
}

pub trait DB {
    fn save_word(&self, word: &str, explain: &str) -> impl Future<Output = Result<(), D1Error>>;
    fn delete_word(&self, word: &str) -> impl Future<Output = Result<(), D1Error>>;
    fn random_word(&self) -> impl Future<Output = Result<Word, D1Error>>;
    fn create_table(&self) -> impl Future<Output = Result<(), D1Error>>;
}

pub enum SQL<'a> {
    CreateTable,
    SaveWord(&'a str, &'a str),
    DeleteWord(&'a str),
    RandomNotRemind,
    Random,
    UpdateRemindTime(&'a str),
}

impl<'a> SQL<'a> {
    pub fn sql(&self) -> &str {
        match self {
            SQL::SaveWord(_, _) => {
                "INSERT INTO words (word, explain, add_time, update_time) VALUES (?, ?, strftime('%s', 'now'), strftime('%s', 'now')) ON CONFLICT(word) DO UPDATE SET explain = ?, update_time = strftime('%s', 'now')"
            }
            SQL::DeleteWord(_) => "DELETE FROM words WHERE word = ?",
            SQL::RandomNotRemind => {
                "SELECT * FROM words WHERE reminder_time <= strftime('%s', 'now') - 43200 ORDER BY RANDOM() LIMIT 1"
            }
            SQL::Random => "SELECT * FROM words ORDER BY RANDOM() LIMIT 1",
            SQL::UpdateRemindTime(_) => {
                "UPDATE words SET reminder_time = strftime('%s', 'now') WHERE word = ?"
            }
            SQL::CreateTable => {
                r#"
CREATE TABLE IF NOT EXISTS [words] (
    "word" TEXT PRIMARY KEY,
    "explain" TEXT,
    "add_time" INTEGER,
    "update_time" INTEGER,
    "reminder_time" INTEGER DEFAULT 0
);
               "#
            }
        }
    }

    pub fn params<T: From<String>>(&self) -> Vec<T> {
        match self {
            SQL::SaveWord(word, explain) => {
                vec![
                    (*word).to_string().into(),
                    (*explain).to_string().into(),
                    (*explain).to_string().into(),
                ]
            }
            SQL::DeleteWord(word) => {
                vec![(*word).to_string().into()]
            }
            SQL::UpdateRemindTime(word) => {
                vec![(*word).to_string().into()]
            }
            SQL::RandomNotRemind | SQL::Random | SQL::CreateTable => {
                vec![]
            }
        }
    }
}
