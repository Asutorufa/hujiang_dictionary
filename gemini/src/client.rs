use crate::model::*;
use futures_util::Stream;
use reqwest::{Client as HttpClient, Url};
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl Client {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(),
            api_key: api_key.into(),
            model: model.into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    fn url(&self, action: &str) -> String {
        format!("{}/models/{}:{}", self.base_url, self.model, action)
    }

    pub async fn generate_content(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Error> {
        let mut url =
            Url::parse(&self.url("generateContent")).map_err(|e| Error::Api(e.to_string()))?;
        url.query_pairs_mut().append_pair("key", &self.api_key);

        let resp = self.http.post(url).json(request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(Error::Api(error_text));
        }

        let response: GenerateContentResponse = resp.json().await?;
        Ok(response)
    }

    pub async fn stream_generate_content(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, Error>>, Error> {
        let mut url = Url::parse(&self.url("streamGenerateContent"))
            .map_err(|e| Error::Api(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("alt", "sse");

        let resp = self.http.post(url).json(request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(Error::Api(error_text));
        }

        let stream = resp.bytes_stream();
        Ok(SseStream {
            inner: stream,
            buffer: String::new(),
        })
    }

    // Updated to take ownership of self and request
    pub async fn stream_generate_content_owned(
        self,
        request: GenerateContentRequest,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, Error>>, Error> {
        let mut url = Url::parse(&self.url("streamGenerateContent"))
            .map_err(|e| Error::Api(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("alt", "sse");

        let resp = self.http.post(url).json(&request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(Error::Api(error_text));
        }

        let stream = resp.bytes_stream();
        Ok(SseStream {
            inner: stream,
            buffer: String::new(),
        })
    }
}

pub struct SseStream<S> {
    inner: S,
    buffer: String,
}

impl<S> Stream for SseStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<GenerateContentResponse, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(pos) = self.buffer.find("\n\n") {
                let message = self.buffer[..pos].to_string();
                self.buffer.drain(..pos + 2);

                if let Some(data) = message.strip_prefix("data: ") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        return Poll::Ready(None);
                    }
                    if !data.is_empty() {
                        match serde_json::from_str::<GenerateContentResponse>(data) {
                            Ok(response) => return Poll::Ready(Some(Ok(response))),
                            Err(e) => return Poll::Ready(Some(Err(Error::Serialization(e)))),
                        }
                    }
                }
                continue;
            }

            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&chunk));
                    continue;
                }
                Poll::Ready(Some(Err(e))) => {
                    log::error!("HTTP error in SSE stream: {}", e);
                    return Poll::Ready(Some(Err(Error::Http(e))));
                }
                Poll::Ready(None) => {
                    if !self.buffer.is_empty() {
                        let message = self.buffer.clone();
                        self.buffer.clear();
                        if let Some(data) = message.strip_prefix("data: ") {
                            let data = data.trim();
                            if !data.is_empty() && data != "[DONE]" {
                                match serde_json::from_str::<GenerateContentResponse>(data) {
                                    Ok(response) => return Poll::Ready(Some(Ok(response))),
                                    Err(e) => {
                                        return Poll::Ready(Some(Err(Error::Serialization(e))));
                                    }
                                }
                            }
                        }
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::StreamExt;
    use mockito::Server;

    #[tokio::test]
    async fn test_generate_content() {
        let mut server = Server::new_async().await;
        
        let response_body = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "Hello world"
                            }
                        ],
                        "role": "model"
                    },
                    "finishReason": "STOP",
                    "index": 0,
                    "safetyRatings": []
                }
            ],
            "promptFeedback": {
                "safetyRatings": []
            }
        }"#;

        let mock = server.mock("POST", "/models/gemini-1.5-flash:generateContent?key=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create_async()
            .await;

        let client = Client::new("test_key", "gemini-1.5-flash").with_base_url(server.url());
        
        let request = GenerateContentRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("Say hello".to_string()),
                    inline_data: None,
                    thought: None,
                }],
            }],
            tools: None,
            safety_settings: None,
            system_instruction: None,
            generation_config: None,
        };

        let response = client.generate_content(&request).await.expect("Failed to generate content");
        
        mock.assert_async().await;
        
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(
            response.candidates[0].content.parts[0].text.as_deref(),
            Some("Hello world")
        );
    }

    #[tokio::test]
    async fn test_stream_generate_content() {
        let mut server = Server::new_async().await;
        
        let sse_body = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hello\"}],\"role\":\"model\"}}]}\n\n\
                        data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\" world\"}],\"role\":\"model\"}}]}\n\n\
                        data: [DONE]\n\n";

        let mock = server.mock("POST", "/models/gemini-1.5-flash:streamGenerateContent?key=test_key&alt=sse")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(sse_body)
            .create_async()
            .await;

        let client = Client::new("test_key", "gemini-1.5-flash").with_base_url(server.url());
        
        let request = GenerateContentRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some("Say hello".to_string()),
                    inline_data: None,
                    thought: None,
                }],
            }],
            tools: None,
            safety_settings: None,
            system_instruction: None,
            generation_config: None,
        };

        let stream = client.stream_generate_content(&request).await.expect("Failed to create stream");
        
        let results: Vec<_> = stream.collect().await;
        mock.assert_async().await;

        assert_eq!(results.len(), 2);
        
        let first = results[0].as_ref().unwrap();
        assert_eq!(
            first.candidates[0].content.parts[0].text.as_deref(),
            Some("Hello")
        );

        let second = results[1].as_ref().unwrap();
        assert_eq!(
            second.candidates[0].content.parts[0].text.as_deref(),
            Some(" world")
        );
    }
}
