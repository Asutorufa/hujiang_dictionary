use std::collections::HashSet;

#[derive(Clone)]
pub struct RunOpt<T: crate::d1::DB, T2: crate::ai::AI> {
    pub allow_users: HashSet<u64>,
    pub matainer: u64,
    pub d1: T,
    pub workers_ai: T2,
}
