use std::collections::HashSet;
use std::sync::Arc;
use std::sync::RwLock;

use frankenstein::client_reqwest;

use d1_orm::DatabaseExecutor;

pub struct ConfigCache {
    pub allow_users: Arc<HashSet<i64>>,
    pub maintainer_id: i64,
    pub bot: Option<client_reqwest::Bot>,
    pub last_updated: u64,
}

#[derive(Clone)]
pub struct RunOpt<T: DatabaseExecutor, T2: crate::ai::Translator> {
    pub d1: T,
    pub translator: T2,
    pub workers_ai: Option<hj_ai::provider::Provider>,
    pub auth_secret: String,
    pub auth_username: String,
    pub auth_password: String,
    pub auth_token_expiration: i64,
    pub config_cache: Arc<RwLock<ConfigCache>>,
}

impl<T: DatabaseExecutor, T2: crate::ai::Translator> RunOpt<T, T2> {
    pub async fn get_config(&self) -> Result<ConfigCache, Box<dyn std::error::Error>> {
        let now = chrono::Utc::now().timestamp() as u64;
        {
            let cache = self.config_cache.read().unwrap();
            if now - cache.last_updated < 300 {
                return Ok(ConfigCache {
                    allow_users: cache.allow_users.clone(),
                    maintainer_id: cache.maintainer_id,
                    bot: cache.bot.clone(),
                    last_updated: cache.last_updated,
                });
            }
        }

        // Fetch from database
        let configurations: Vec<crate::d1::Configuration> = self
            .d1
            .query_all(crate::d1::Queries::ListConfigurations)
            .await?;

        let mut allow_users_str = "".to_string();
        let mut maintainer_id = 0;
        let mut telegram_token = "".to_string();

        for config in configurations {
            match config.key.as_str() {
                "ALLOW_USERS" => allow_users_str = config.value,
                "MAINTAINER_ID" => maintainer_id = config.value.parse::<i64>().unwrap_or(0),
                "TELEGRAM_TOKEN" => telegram_token = config.value,
                _ => {}
            }
        }

        let mut set = HashSet::from([maintainer_id]);
        for v in allow_users_str.split(",") {
            if let Ok(id) = v.parse::<i64>() {
                set.insert(id);
            }
        }

        let bot = if telegram_token.is_empty() {
            None
        } else {
            Some(client_reqwest::Bot::new(&telegram_token))
        };

        let mut cache = self.config_cache.write().unwrap();
        cache.allow_users = Arc::new(set);
        cache.maintainer_id = maintainer_id;
        cache.bot = bot;
        cache.last_updated = now;

        Ok(ConfigCache {
            allow_users: cache.allow_users.clone(),
            maintainer_id: cache.maintainer_id,
            bot: cache.bot.clone(),
            last_updated: cache.last_updated,
        })
    }
}
