use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

pub trait Completion {
    fn completion(
        &self,
        messages: Vec<Message>,
    ) -> impl Future<Output = Result<CompletionResponse, String>> + Send;

    fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> impl Future<
        Output = Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String>,
    > + Send;
}
