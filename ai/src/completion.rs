use futures_util::Stream;
use serde::{Serialize, Deserialize};
use std::future::Future;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub trait Completion {
    fn completion(
        &self,
        messages: Vec<Message>,
    ) -> impl Future<Output = Result<String, String>> + Send;

    fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> impl Future<Output = Result<impl Stream<Item = Result<String, String>> + Send, String>> + Send;
}
