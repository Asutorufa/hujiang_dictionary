use serde::{Deserialize, Serialize};
use std::fmt;
use value2struct::FromValueVec;

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Word {
    pub word: String,
    pub explain: String,
    add_time: i64,
    update_time: i64,
    reminder_time: i64,
}

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Empty {}

#[derive(Debug)]
pub struct Error(pub String);

unsafe impl Send for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error(s.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

pub trait DB {
    fn save_word(&self, word: &str, explain: &str) -> impl Future<Output = Result<(), Error>>;
    fn delete_word(&self, word: &str) -> impl Future<Output = Result<(), Error>>;
    fn random_word(&self) -> impl Future<Output = Result<Word, Error>>;
    fn list_word(
        &self,
        page_size: u64,
        page_number: u64,
    ) -> impl Future<Output = Result<Vec<Word>, Error>>;
    fn create_table(&self) -> impl Future<Output = Result<(), Error>>;
}

pub trait DBv2 {
    fn exec<T>(&self, sql: SQL<'_>) -> impl Future<Output = Result<Vec<T>, Error>>
    where
        T: for<'a> Deserialize<'a>;

    fn save_word(&self, word: &str, explain: &str) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::SaveWord(word, explain)).await?;
            Ok(())
        }
    }

    fn delete_word(&self, word: &str) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::DeleteWord(word)).await?;
            Ok(())
        }
    }

    fn list_word(
        &self,
        page_size: u64,
        page_number: u64,
    ) -> impl Future<Output = Result<Vec<Word>, Error>> {
        async move {
            self.exec::<Word>(SQL::ListWord(page_size, page_number))
                .await
        }
    }

    fn random_word(&self) -> impl Future<Output = Result<Word, Error>> {
        async move {
            let words = match self.exec::<Word>(SQL::RandomNotRemind).await {
                Ok(v) if !v.is_empty() => v[0].clone(),
                _ => self
                    .exec::<Word>(SQL::Random)
                    .await?
                    .first()
                    .ok_or(Error("no word found".to_string()))?
                    .clone(),
            };

            let update_sql = SQL::UpdateRemindTime(words.word.as_ref());

            self.exec::<Empty>(update_sql).await?;

            Ok(words)
        }
    }

    fn create_table(&self) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::CreateTable).await?;
            Ok(())
        }
    }
}

pub enum SQL<'a> {
    CreateTable,
    SaveWord(&'a str, &'a str),
    DeleteWord(&'a str),
    RandomNotRemind,
    Random,
    UpdateRemindTime(&'a str),
    ListWord(u64, u64),
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
            SQL::ListWord(_, _) => "SELECT * FROM words ORDER BY word LIMIT ? OFFSET ?",
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
            SQL::ListWord(page_size, page_number) => {
                let size = if *page_size > 0 { 10 } else { *page_size };
                let offset = (*page_number - 1) * size;

                vec![size.to_string().into(), offset.to_string().into()]
            }
        }
    }
}
