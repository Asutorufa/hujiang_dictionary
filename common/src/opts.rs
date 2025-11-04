use std::collections::{HashMap, HashSet};

use frankenstein::client_reqwest;

use crate::ai::OpenAI;

#[derive(Clone)]
pub struct RunOpt<T: crate::d1::DB, T2: crate::ai::WorkersAI> {
    pub allow_users: HashSet<i64>,
    pub matainer: i64,
    pub d1: T,
    pub workers_ai: T2,
    pub bot: client_reqwest::Bot,
    pub custom_llms: HashMap<String, OpenAI>,
}
