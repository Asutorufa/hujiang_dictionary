use crate::d1::WasmD1;
use hjcommon::d1::{DB, Empty, Error as D1Error, SQL, Word};

impl DB for WasmD1 {
    async fn create_table(&self) -> Result<(), D1Error> {
        self.exec::<Empty>(SQL::CreateTable).await?;
        Ok(())
    }

    async fn delete_word(&self, word: &str) -> Result<(), D1Error> {
        let sql = SQL::DeleteWord(word);
        self.exec::<Empty>(sql).await?;
        Ok(())
    }

    async fn random_word(&self) -> Result<Word, D1Error> {
        let words = match self.exec::<Word>(SQL::RandomNotRemind).await {
            Ok(v) if !v.is_empty() => v[0].clone(),
            _ => self
                .exec::<Word>(SQL::Random)
                .await?
                .first()
                .ok_or(D1Error("no word found".to_string()))?
                .clone(),
        };

        let update_sql = SQL::UpdateRemindTime(words.word.as_ref());

        self.exec::<Empty>(update_sql).await?;

        Ok(words)
    }

    async fn save_word(&self, word: &str, explain: &str) -> Result<(), D1Error> {
        let sql = SQL::SaveWord(None, word, explain, 0);
        self.exec::<Empty>(sql).await?;
        Ok(())
    }

    async fn list_word(
        &self,
        page_size: u64,
        page_number: u64,
        order_by: &str,
    ) -> Result<Vec<Word>, D1Error> {
        self.exec::<Word>(SQL::ListWord(page_size, page_number, order_by, 0))
            .await
    }
}
