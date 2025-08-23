use std::collections::HashSet;

use frankenstein::client_reqwest;

#[derive(Clone)]
pub struct RunOpt<T: crate::d1::DBv2, T2: crate::ai::AI> {
    pub allow_users: HashSet<i64>,
    pub matainer: i64,
    pub d1: T,
    pub workers_ai: T2,
    pub bot: client_reqwest::Bot,
}
