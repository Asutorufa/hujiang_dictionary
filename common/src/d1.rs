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
    anki_count: u64,
    priority: u64,
    r#type: u64,
}

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Count {
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Empty {}

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct ColumnExist {
    pub exist: u32,
}

#[derive(Debug)]
pub struct Error(pub String);

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
    fn exec<T>(&self, sql: SQL<'_>) -> impl Future<Output = Result<Vec<T>, Error>>
    where
        T: for<'a> Deserialize<'a>;

    fn save_word(
        &self,
        origin: Option<&str>,
        word: &str,
        explain: &str,
        r#type: i64,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::SaveWord(origin, word, explain, r#type))
                .await?;
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
        order_by: &str,
        r#type: i64,
    ) -> impl Future<Output = Result<Vec<Word>, Error>> {
        async move {
            self.exec::<Word>(SQL::ListWord(page_size, page_number, order_by, r#type))
                .await
        }
    }

    fn count_word(&self, r#type: i64) -> impl Future<Output = Result<u64, Error>> {
        async move {
            let c = self.exec::<Count>(SQL::CountWord(r#type)).await?;

            if c.is_empty() {
                Err(Error("get count failed".to_string()))
            } else {
                Ok(c[0].size)
            }
        }
    }

    fn increment_remind_count(&self, word: &str) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::IncrementRemindCount(word)).await?;
            Ok(())
        }
    }

    fn change_priority(
        &self,
        word: &str,
        priority: u64,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::ChangePriority(word, priority))
                .await?;
            Ok(())
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

    fn check_column_exists(&self, column: &str) -> impl Future<Output = Result<bool, Error>> {
        async move {
            let c = self
                .exec::<ColumnExist>(SQL::CheckColumnExists(column))
                .await?;
            if c.is_empty() {
                return Ok(false);
            }

            Ok(c[0].exist == 1)
        }
    }

    fn add_column(&self, columns: Vec<(&str, &str)>) -> impl Future<Output = Result<(), Error>> {
        async move {
            for (column, r#type) in columns {
                if self.check_column_exists(column).await? {
                    continue;
                }

                self.exec::<Empty>(SQL::AddColumn(column, r#type)).await?;
            }
            Ok(())
        }
    }

    fn create_table(&self) -> impl Future<Output = Result<(), Error>> {
        async move {
            self.exec::<Empty>(SQL::CreateTable).await?;

            /*
                ALTER TABLE words ADD COLUMN IF NOT EXISTS anki_count INTEGER DEFAULT 1;
                ALTER TABLE words ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0;
                ALTER TABLE words ADD COLUMN IF NOT EXISTS type INTEGER DEFAULT 0;
            */
            self.add_column(vec![
                ("anki_count", "INTEGER DEFAULT 1"),
                ("priority", "INTEGER DEFAULT 0"),
                ("type", "INTEGER DEFAULT 0"),
            ])
            .await?;

            Ok(())
        }
    }
}

pub enum SQL<'a> {
    CreateTable,
    RandomNotRemind,
    Random,
    UpdateRemindTime(&'a str),
    SaveWord(Option<&'a str>, &'a str, &'a str, i64),
    DeleteWord(&'a str),
    ListWord(u64, u64, &'a str, i64),
    CountWord(i64),
    IncrementRemindCount(&'a str),
    ChangePriority(&'a str, u64),

    CheckColumnExists(&'a str),
    AddColumn(&'a str, &'a str),
}

impl<'a> SQL<'a> {
    pub fn sql(&self) -> String {
        match self {
            SQL::RandomNotRemind => {
                "SELECT * FROM words WHERE reminder_time <= strftime('%s', 'now') - 43200 ORDER BY RANDOM() LIMIT 1".to_string()
            }
            SQL::Random => "SELECT * FROM words ORDER BY RANDOM() LIMIT 1".to_string(),
            SQL::UpdateRemindTime(_) => {
                "UPDATE words SET reminder_time = strftime('%s', 'now') WHERE word = ?".to_string()
            }
            SQL::CreateTable => {
                r#"
CREATE TABLE IF NOT EXISTS [words] (
    "word" TEXT PRIMARY KEY,
    "explain" TEXT,
    "add_time" INTEGER,
    "update_time" INTEGER,
    "reminder_time" INTEGER DEFAULT 0,
    "anki_count" INTEGER DEFAULT 1,
    "priority" INTEGER DEFAULT 0,
    -- 0: word, 1: grammar
    "type" INTEGER DEFAULT 0
);
CREATE TABLE IF NOT EXISTS [configurations] (
    "key" TEXT PRIMARY KEY,
    "value" TEXT
);
"#.to_string()
            }

            SQL::SaveWord(origin, now, _, _) =>   match origin {
                    Some(word) if word!= now=>"UPDATE words SET word = ?, explain = ?, update_time = strftime('%s', 'now'), type = ? WHERE word = ?".to_string(),
                    _ => r#"
                        INSERT INTO words (word, explain, add_time, update_time, type) 
                        VALUES (?, ?, strftime('%s', 'now'), strftime('%s', 'now'), ?) 
                        ON CONFLICT(word) DO UPDATE SET explain = ?, update_time = strftime('%s', 'now'), type = ?
                    "#.to_string(),
                }
            SQL::DeleteWord(_) => "DELETE FROM words WHERE word = ?".to_string(),
            SQL::ListWord(_, _, order_by, _) => format!("SELECT * FROM words WHERE type = ? ORDER BY {} LIMIT ? OFFSET ?", order_by),
            SQL::CountWord(_) => "SELECT count(*) as size FROM words WHERE type = ?".to_string(),
            SQL::IncrementRemindCount(_) => {
                "UPDATE words SET anki_count = anki_count + 1 WHERE word = ?".to_string()
            }
            SQL::ChangePriority(_, _) => "UPDATE words SET priority = ? WHERE word = ?".to_string(),

            SQL::CheckColumnExists(column) => {
                format!(r#"
SELECT CASE 
    WHEN EXISTS (SELECT 1 FROM pragma_table_info('words') WHERE name='{}') 
    THEN 1 ELSE 0 
END AS exist;
"#, column)
            }
            SQL::AddColumn(column,r#type) => {
                format!("ALTER TABLE words ADD COLUMN {} {}", column,r#type)
            }
        }
    }

    pub fn params<T: From<String>>(&self) -> Vec<T> {
        match self {
            SQL::SaveWord(origin, word, explain, r#type) => match origin {
                Some(origin) if origin != word => vec![
                    (*word).to_string().into(),
                    (*explain).to_string().into(),
                    (*r#type).to_string().into(),
                    (*origin).to_string().into(),
                ],
                _ => vec![
                    (*word).to_string().into(),
                    (*explain).to_string().into(),
                    (*r#type).to_string().into(),
                    (*explain).to_string().into(),
                    (*r#type).to_string().into(),
                ],
            },

            SQL::DeleteWord(word) => {
                vec![(*word).to_string().into()]
            }
            SQL::UpdateRemindTime(word) => {
                vec![(*word).to_string().into()]
            }
            SQL::ListWord(page_size, page_number, _, r#type) => {
                let size = if *page_size > 0 { 10 } else { *page_size };
                let offset = (*page_number - 1) * size;

                vec![
                    (*r#type).to_string().into(),
                    size.to_string().into(),
                    offset.to_string().into(),
                ]
            }
            SQL::CountWord(r#type) => vec![(*r#type).to_string().into()],
            SQL::IncrementRemindCount(word) => vec![(*word).to_string().into()],

            SQL::ChangePriority(word, priority) => {
                vec![(*priority).to_string().into(), (*word).to_string().into()]
            }

            SQL::CheckColumnExists(_)
            | SQL::AddColumn(_, _)
            | SQL::RandomNotRemind
            | SQL::Random
            | SQL::CreateTable => vec![],
        }
    }
}
