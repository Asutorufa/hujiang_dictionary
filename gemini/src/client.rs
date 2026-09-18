use crate::model::*;
use futures_util::Stream;
use reqwest::{Client as HttpClient, Url};
use serde::Deserialize;
use std::collections::HashSet;
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
    api_key: Option<String>,
    token: Option<String>,
    model: String,
    base_url: String,
    project_id: Option<String>,
    location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListModelsResponse {
    #[serde(default)]
    models: Vec<ListModel>,
    #[serde(default)]
    publisher_models: Vec<ListModel>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListModel {
    name: String,
    base_model_id: Option<String>,
    #[serde(default)]
    supported_generation_methods: Vec<String>,
    #[serde(default)]
    supported_actions: Vec<String>,
}

impl Client {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(),
            api_key: Some(api_key.into()),
            token: None,
            model: model.into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            project_id: None,
            location: None,
        }
    }

    pub fn new_vertex_ai(
        project_id: impl Into<String>,
        location: impl Into<String>,
        model: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        let location = location.into();
        Self {
            http: HttpClient::new(),
            api_key: None,
            token: Some(token.into()),
            model: model.into(),
            base_url: format!("https://{}-aiplatform.googleapis.com/v1", location),
            project_id: Some(project_id.into()),
            location: Some(location),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    fn models_url(&self) -> String {
        if let (Some(project_id), Some(location)) = (&self.project_id, &self.location) {
            format!(
                "{}/projects/{}/locations/{}/publishers/google/models",
                self.base_url.trim_end_matches('/'),
                project_id,
                location
            )
        } else {
            format!("{}/models", self.base_url.trim_end_matches('/'))
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>, Error> {
        let models_url = self.models_url();
        let mut page_token: Option<String> = None;
        let mut models = Vec::new();
        let mut seen = HashSet::new();

        loop {
            let mut query = vec![("pageSize", "1000")];
            if let Some(api_key) = &self.api_key {
                query.push(("key", api_key));
            }
            if let Some(page_token) = &page_token {
                query.push(("pageToken", page_token));
            }

            let mut request = self.http.get(&models_url).query(&query);
            if let Some(token) = &self.token {
                request = request.bearer_auth(token);
            }

            let response = request.send().await?;
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await?;
                return Err(Error::Api(error_text));
            }

            let page: ListModelsResponse = response.json().await?;
            for model in page.models.into_iter().chain(page.publisher_models) {
                let supports_generate_content = model.supported_generation_methods.is_empty()
                    && model.supported_actions.is_empty()
                    || model
                        .supported_generation_methods
                        .iter()
                        .chain(model.supported_actions.iter())
                        .any(|action| action == "generateContent");
                if !supports_generate_content {
                    continue;
                }

                let model_id = model
                    .base_model_id
                    .filter(|model_id| !model_id.trim().is_empty())
                    .unwrap_or_else(|| {
                        model
                            .name
                            .rsplit('/')
                            .next()
                            .unwrap_or_default()
                            .to_string()
                    });
                let model_id = model_id.strip_prefix("models/").unwrap_or(&model_id).trim();
                if !model_id.is_empty() && seen.insert(model_id.to_string()) {
                    models.push(model_id.to_string());
                }
            }

            let Some(next_page_token) = page.next_page_token.filter(|token| !token.is_empty())
            else {
                break;
            };
            if page_token.as_deref() == Some(next_page_token.as_str()) {
                break;
            }
            page_token = Some(next_page_token);
        }

        Ok(models)
    }

    fn url(&self, action: &str) -> String {
        if let Some(project_id) = &self.project_id {
            let location = self.location.as_deref().unwrap_or("us-central1");
            format!(
                "{}/projects/{}/locations/{}/publishers/google/models/{}:{}",
                self.base_url, project_id, location, self.model, action
            )
        } else {
            format!("{}/models/{}:{}", self.base_url, self.model, action)
        }
    }

    pub async fn generate_content(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse, Error> {
        let url =
            Url::parse(&self.url("generateContent")).map_err(|e| Error::Api(e.to_string()))?;

        let mut req_builder = self.http.post(url);

        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.query(&[("key", api_key)]);
        }

        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }

        let resp = req_builder.json(request).send().await?;

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
        let url = Url::parse(&self.url("streamGenerateContent"))
            .map_err(|e| Error::Api(e.to_string()))?;

        let mut req_builder = self.http.post(url);

        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.query(&[("key", api_key)]);
        }

        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }

        let resp = req_builder
            .query(&[("alt", "sse")])
            .json(request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(Error::Api(error_text));
        }

        let stream = bytes_stream(resp);
        Ok(SseStream {
            inner: stream,
            buffer: String::new(),
            queue: VecDeque::new(),
        })
    }

    // Updated to take ownership of self and request
    pub async fn stream_generate_content_owned(
        self,
        request: GenerateContentRequest,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, Error>>, Error> {
        let url = Url::parse(&self.url("streamGenerateContent"))
            .map_err(|e| Error::Api(e.to_string()))?;

        let mut req_builder = self.http.post(url);

        if let Some(api_key) = &self.api_key {
            req_builder = req_builder.query(&[("key", api_key)]);
        }

        if let Some(token) = &self.token {
            req_builder = req_builder.bearer_auth(token);
        }

        let resp = req_builder
            .query(&[("alt", "sse")])
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(Error::Api(error_text));
        }

        let stream = bytes_stream(resp);
        Ok(SseStream {
            inner: stream,
            buffer: String::new(),
            queue: VecDeque::new(),
        })
    }
}

fn bytes_stream(
    response: reqwest::Response,
) -> Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>>>> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::pin(futures_util::stream::once(
            async move { response.bytes().await },
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::pin(response.bytes_stream())
    }
}

use std::collections::VecDeque;

pub struct SseStream<S> {
    inner: S,
    buffer: String,
    queue: VecDeque<GenerateContentResponse>,
}

impl<S> Stream for SseStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<GenerateContentResponse, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(response) = self.queue.pop_front() {
                return Poll::Ready(Some(Ok(response)));
            }

            let mut split_pos = None;
            let mut drain_len = 0;

            if let Some(pos) = self.buffer.find("\n\n") {
                split_pos = Some(pos);
                drain_len = 2;
            }
            if let Some(pos) = self
                .buffer
                .find("\r\n\r\n")
                .filter(|&pos| split_pos.is_none_or(|p| pos < p))
            {
                split_pos = Some(pos);
                drain_len = 4;
            }

            if let Some(pos) = split_pos {
                let message = self.buffer[..pos].to_string();
                self.buffer.drain(..pos + drain_len);

                let mut combined_json = String::new();
                for line in message.lines() {
                    let line = line.trim();
                    if let Some(data) = line.strip_prefix("data:") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            return Poll::Ready(None);
                        }
                        if !data.is_empty() {
                            combined_json.push_str(data);
                            combined_json.push('\n');
                        }
                    }
                }

                if !combined_json.is_empty() {
                    let deserializer = serde_json::Deserializer::from_str(&combined_json);
                    let iter = deserializer.into_iter::<GenerateContentResponse>();

                    for result in iter {
                        match result {
                            Ok(response) => self.queue.push_back(response),
                            Err(e) => {
                                if self.queue.is_empty() {
                                    return Poll::Ready(Some(Err(Error::Serialization(e))));
                                }
                                break;
                            }
                        }
                    }
                }

                if let Some(response) = self.queue.pop_front() {
                    return Poll::Ready(Some(Ok(response)));
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

                        let mut combined_json = String::new();
                        for line in message.lines() {
                            let line = line.trim();
                            if let Some(data) = line.strip_prefix("data:") {
                                let data = data.trim();
                                if data == "[DONE]" {
                                    return Poll::Ready(None);
                                }
                                if !data.is_empty() {
                                    combined_json.push_str(data);
                                    combined_json.push('\n');
                                }
                            }
                        }

                        if !combined_json.is_empty() {
                            let deserializer = serde_json::Deserializer::from_str(&combined_json);
                            let iter = deserializer.into_iter::<GenerateContentResponse>();
                            for result in iter {
                                match result {
                                    Ok(response) => self.queue.push_back(response),
                                    Err(e) => {
                                        if self.queue.is_empty() {
                                            return Poll::Ready(Some(Err(Error::Serialization(e))));
                                        }
                                        break;
                                    }
                                }
                            }
                        }

                        if let Some(response) = self.queue.pop_front() {
                            return Poll::Ready(Some(Ok(response)));
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
    async fn test_list_models() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/models?pageSize=1000&key=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"models":[{"name":"models/gemini-2.5-flash","baseModelId":"gemini-2.5-flash","supportedGenerationMethods":["generateContent"]},{"name":"models/text-embedding-005","supportedGenerationMethods":["embedContent"]}]}"#,
            )
            .create_async()
            .await;

        let client = Client::new("test_key", "gemini-2.5-flash").with_base_url(server.url());

        assert_eq!(
            client.list_models().await.unwrap(),
            vec!["gemini-2.5-flash".to_string()]
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_vertex_ai_models() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock(
                "GET",
                "/projects/test-project/locations/us-central1/publishers/google/models?pageSize=1000",
            )
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"publisherModels":[{"name":"publishers/google/models/gemini-2.5-pro","baseModelId":"gemini-2.5-pro","supportedActions":["generateContent"]}]}"#,
            )
            .create_async()
            .await;

        let client = Client::new_vertex_ai(
            "test-project",
            "us-central1",
            "gemini-2.5-pro",
            "test_token",
        )
        .with_base_url(server.url());

        assert_eq!(
            client.list_models().await.unwrap(),
            vec!["gemini-2.5-pro".to_string()]
        );
        mock.assert_async().await;
    }

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

        let mock = server
            .mock(
                "POST",
                "/models/gemini-1.5-flash:generateContent?key=test_key",
            )
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

        let response = client
            .generate_content(&request)
            .await
            .expect("Failed to generate content");

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

        let mock = server
            .mock(
                "POST",
                "/models/gemini-1.5-flash:streamGenerateContent?key=test_key&alt=sse",
            )
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

        let stream = client
            .stream_generate_content(&request)
            .await
            .expect("Failed to create stream");

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

    #[tokio::test]
    async fn test_vertex_ai_generate_content() {
        let mut server = Server::new_async().await;

        let response_body = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "Hello from Vertex AI"
                            }
                        ],
                        "role": "model"
                    }
                }
            ]
        }"#;

        let project_id = "test-project";
        let location = "us-central1";
        let model = "gemini-1.5-flash";
        let path = format!(
            "/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            project_id, location, model
        );

        let mock = server
            .mock("POST", path.as_str())
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create_async()
            .await;

        let client = Client::new_vertex_ai(project_id, location, model, "test_token")
            .with_base_url(server.url());

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

        let response = client
            .generate_content(&request)
            .await
            .expect("Failed to generate content");

        mock.assert_async().await;

        assert_eq!(response.candidates.len(), 1);
        assert_eq!(
            response.candidates[0].content.parts[0].text.as_deref(),
            Some("Hello from Vertex AI")
        );
    }
}
