use d1_orm::*;
use serde::{Deserialize, Serialize};

// Re-export needed types
pub use d1_orm::Error;
use d1_orm::MigrationInfo;

define_model!(
    Word,
    WordField,
    WordUpdate {
        word: String[pk],
        explain: String,
        example: String,
        add_time: i64,
        update_time: i64,
        reminder_time: i64,
        anki_count: u64,
        priority: u64,
        #[serde(rename(serialize = "type"))]
        word_type: i64,
        rand_key: i64,
    }
);

define_model!(
    Configuration,
    ConfigurationField,
    ConfigurationUpdate {
        key: String[pk],
        value: String,
    }
);

define_sql!(
    Queries

    RandomNotRemind => "SELECT * FROM words WHERE reminder_time <= strftime('%s', 'now') - 43200 AND rand_key >= abs(random()) ORDER BY rand_key LIMIT 1",
    Random => "SELECT * FROM words WHERE rand_key >= abs(random()) ORDER BY rand_key LIMIT 1",

    UpdateRemindTime { word: &'a str } => "UPDATE words SET reminder_time = strftime('%s', 'now') WHERE word = ?",
    IncrementRemindCount { word: &'a str } => "UPDATE words SET anki_count = anki_count + 1 WHERE word = ?",
    ChangePriority { priority: u64, word: &'a str } => "UPDATE words SET priority = ? WHERE word = ?",

    DeleteWord { word: &'a str } => "DELETE FROM words WHERE word = ?",
    CountWord { word_type: i64 } => "SELECT count(*) as size FROM words WHERE word_type = ?",

    // SaveWord (Upsert)
    SaveWord {
        word: &'a str,
        explain: &'a str,
        word_type: i64,
        example: &'a str
    } => r#"
        INSERT INTO words (word, explain, add_time, update_time, word_type, example, rand_key)
        VALUES (?1, ?2, strftime('%s', 'now'), strftime('%s', 'now'), ?3, ?4, abs(random()))
        ON CONFLICT(word) DO UPDATE SET
            explain = excluded.explain,
            update_time = strftime('%s', 'now'),
            word_type = excluded.word_type,
            example = excluded.example
    "#,

    // Rename (Update PK)
    RenameWord {
        new_word: &'a str,
        explain: &'a str,
        word_type: i64,
        example: &'a str,
        old_word: &'a str
    } => "UPDATE words SET word = ?1, explain = ?2, update_time = strftime('%s', 'now'), word_type = ?3, example = ?4 WHERE word = ?5",
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Count {
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct SqlStatement(pub String);

impl Query for SqlStatement {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        Ok((Cow::Owned(self.0.clone()), vec![]))
    }
}

impl MigrationMeta for SqlStatement {
    fn migration_info(&self) -> Option<MigrationInfo> {
        None
    }
}

pub fn migrations() -> Vec<Migration<SqlStatement>> {
    vec![
        Migration::new(
            1,
            "initial",
            vec![SqlStatement(
                r#"
            CREATE TABLE IF NOT EXISTS words (
                "word" TEXT PRIMARY KEY,
                "explain" TEXT,
                "add_time" INTEGER,
                "update_time" INTEGER,
                "reminder_time" INTEGER DEFAULT 0,
                "anki_count" INTEGER DEFAULT 1,
                "priority" INTEGER DEFAULT 0,
                "type" INTEGER DEFAULT 0,
                "example" TEXT DEFAULT '',
                "rand_key" INTEGER
            );
            CREATE TABLE IF NOT EXISTS configurations (
                "key" TEXT PRIMARY KEY,
                "value" TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_word_key ON words(word);
            CREATE INDEX IF NOT EXISTS idx_type_key ON words(type);
            CREATE INDEX IF NOT EXISTS idx_rand_key ON words(rand_key);
            CREATE INDEX IF NOT EXISTS idx_reminder_time_key ON words(reminder_time);
            CREATE INDEX IF NOT EXISTS idx_type_reminder_time_key ON words(type, reminder_time);
            CREATE INDEX IF NOT EXISTS idx_type_update_time_key ON words(type, update_time);
            CREATE INDEX IF NOT EXISTS idx_type_add_time_key ON words(type, add_time);
            CREATE INDEX IF NOT EXISTS idx_type_priority_key ON words(type, priority);
            CREATE INDEX IF NOT EXISTS idx_words_reminder_rand ON words(reminder_time, rand_key);
            "#
                .to_string(),
            )],
        ),
        Migration::new(
            2,
            "rename_type",
            vec![SqlStatement(
                r#"
            ALTER TABLE words RENAME COLUMN type TO word_type;
            "#
                .to_string(),
            )],
        ),
    ]
}

pub struct RawQuery {
    pub sql: String,
    pub params: Vec<DatabaseValue>,
}

use std::borrow::Cow;

impl Query for RawQuery {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        Ok((Cow::Owned(self.sql.clone()), self.params.clone()))
    }
}

// Helper to construct dynamic List query
pub fn list_word_query(
    page_size: u64,
    page_number: u64,
    order_by: &str,
    is_desc: bool,
    word_type: i64,
) -> RawQuery {
    let limit = if page_size > 0 { page_size } else { 10 };
    let offset = (page_number.max(1) - 1) * limit;

    // Validate order_by to prevent injection (though route.rs already does checks, good to be safe)
    let safe_order_by = match order_by {
        "word" | "update_time" | "priority" | "reminder_time" | "anki_count" | "add_time" => {
            order_by
        }
        _ => "word",
    };

    let sql = format!(
        "SELECT * FROM words WHERE word_type = ? ORDER BY {}{} LIMIT ? OFFSET ?",
        safe_order_by,
        if is_desc { " DESC" } else { "" }
    );
    let params = vec![
        DatabaseValue::from(word_type),
        DatabaseValue::from(limit as i64),
        DatabaseValue::from(offset as i64),
    ];

    RawQuery { sql, params }
}

pub async fn random_word(db: &impl DatabaseExecutor) -> Result<Word, Box<dyn std::error::Error>> {
    let word: Option<Word> = db.query_first(Queries::RandomNotRemind).await?;
    let word = match word {
        Some(v) => v,
        None => db
            .query_first(Queries::Random)
            .await?
            .ok_or("no word found")?,
    };

    db.execute(Queries::UpdateRemindTime { word: &word.word })
        .await?;

    Ok(word)
}
