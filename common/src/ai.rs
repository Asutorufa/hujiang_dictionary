use base64::Engine;
use hjdict::google_search::{self, Body};
use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub content: String,
    pub reasoning: Option<String>,
}

impl ToString for Response {
    fn to_string(&self) -> String {
        if let Some(reasoning) = &self.reasoning {
            format!("reasoning:\n{}\ncontent:\n{}", reasoning, self.content)
        } else {
            self.content.clone()
        }
    }
}

pub trait WorkersAI {
    fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> impl Future<Output = Result<String, Error>>;

    fn completion(
        &self,
        req: CompletionRequest,
    ) -> impl Future<Output = Result<CompletionResponse, Error>>;

    fn responses(
        &self,
        req: ResponsesRequest,
    ) -> impl Future<Output = Result<ResponseResponse, Error>>;

    fn translate(
        &self,
        model: Models,
        chars_limit: bool,
        prompt: &str,
        instruction: Option<&str>,
    ) -> impl Future<Output = Result<Response, Error>> {
        async move {
            let req = TranslateRequest {
                query: prompt,
                instruction,
                model: model.as_str(),
                dst_lang: None,
                chars_limit,
            };

            match model {
                Models::Gemma3_12bIt | Models::Llama4Scout17B16EInstruct => {
                    let result = self.completion(req.completion_request()).await?;

                    match result.choices.len() {
                        0 => Err(Error("no choice".to_string())),
                        _ => Ok(result.choices.first().unwrap().message.to_response()),
                    }
                }

                Models::GPTOss20B => {
                    let result = self.responses(req.responses_request()).await?.content();
                    Ok(result)
                }
                _ => Err(Error("model not supported".to_string())),
            }
        }
    }

    fn google_search(
        &self,
        model: Models,
        chars_limit: bool,
        query: &str,
    ) -> impl Future<Output = Result<Response, Error>> {
        async move {
            let instruction = google_search(query).await?;

            let req = TranslateRequest {
                model: model.as_str(),
                query,
                chars_limit,
                instruction: Some(&instruction),
                dst_lang: None,
            };

            info!("google search instruct: {}", instruction);

            match model {
                Models::Gemma3_12bIt | Models::Llama4Scout17B16EInstruct => {
                    let result = self.completion(req.completion_request()).await?;

                    match result.choices.len() {
                        0 => Err(Error("no choice".to_string())),
                        _ => Ok(result.choices.first().unwrap().message.to_response()),
                    }
                }

                Models::GPTOss20B | _ => {
                    let result = self.responses(req.responses_request()).await?.content();

                    Ok(result)
                }
            }
        }
    }
}

async fn google_search(query: &str) -> Result<String, Error> {
    let result = match google_search::getv2(query).await {
        Ok(v) => v,
        Err(e) => return Err(Error::from(e.to_string())),
    };

    let mut instruct = "<search_result>\n".to_string();

    let mut length = if result.len() > 5 { 5 } else { result.len() };

    for v in &result {
        match v {
            Body::Content(s) => {
                instruct.push_str(&format!("<content>{}</content>\n", s));
            }
            Body::Link(v) => {
                if length <= 0 {
                    continue;
                }

                length -= 1;
                let (title, content) = v.get_raw_page().await?;
                instruct.push_str(&format!("\n<content title='{}' link='{}'>\n", title, v.url));
                instruct.push_str(&content);
                instruct.push_str("</content>\n");
            }
        }
    }

    instruct.push_str("\n</search_result>\n");

    instruct.push_str("\n**please use above search result to explain.**\n");

    Ok(instruct)
}

pub static SYSTEM_MSG: &str = r#"You are a professional translator.  

**Note**:  

- Translate or explain the input text according to the instructions.  
- If target language is not specified, return the result in the user’s original language.  
- **Don't use the index of search result**.  

**Translate Rules**:  

- Please present content in a clear and easy-to-understand way.  
- Explain concepts alongside examples whenever possible.  
- For long or complex content, use tables or other formats for clarity.  
- If you have a better way to present the information, feel free to use it.  
"#;

pub static SYSTEM_MSG_LIMIT: &str = r#"You are a professional translator.

**Note**:  

- Translate or explain the input text according to the instructions.  
- If target language is not specified, return the result in the user’s original language.  
- **Don't use the index of search result**.  

**Translate Rules**:  

- Please present content in a clear and easy-to-understand way.  
- Explain concepts alongside examples whenever possible.  
- If you have a better way to present the information, feel free to use it.  
- If the translated content is approaching the limit, prioritize preserving core meaning and compress the expression when necessary. Paraphrase or summarize if required.
- Output in plain text format. **Do not output in Markdown format!**
- The total output must not exceed 4096 characters, including spaces and line breaks.
- Strictly follow the character limit to prevent truncation.
"#;

pub fn system_msg(limit: bool) -> &'static str {
    if limit { SYSTEM_MSG_LIMIT } else { SYSTEM_MSG }
}

#[derive(Debug, Clone)]
pub enum Models {
    Gemma3_12bIt,
    Llama4Scout17B16EInstruct,
    M2M100_1_2B,
    GPTOss20B,
}

impl Models {
    pub fn as_str(&self) -> &'static str {
        match self {
            Models::Gemma3_12bIt => "@cf/google/gemma-3-12b-it",
            Models::Llama4Scout17B16EInstruct => "@cf/meta/llama-4-scout-17b-16e-instruct",
            Models::M2M100_1_2B => "@cf/meta/m2m100-1.2b",
            Models::GPTOss20B => "@cf/openai/gpt-oss-20b",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<hjdict::google_search::SearchError> for Error {
    fn from(value: hjdict::google_search::SearchError) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesRequest {
    pub instructions: String,
    pub model: String,
    pub input: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
}

impl ResponsesRequest {
    pub fn new_translate_request<'a>(req: TranslateRequest<'a>) -> Self {
        let msgs = vec![Message {
            role: "user".to_string(),
            content: format!(
                "{}\n{}",
                req.instruction.unwrap_or_default(),
                req.query.to_string()
            ),
            reasoning: None,
        }];

        Self {
            instructions: system_msg(req.chars_limit).to_string(),
            model: req.model.to_string(),
            input: msgs,
            reasoning: Some(Reasoning {
                effort: Some("low".to_string()),
                summary: Some("concise".to_string()),
                ..Default::default()
            }),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Reasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseResponse {
    pub output: Vec<ResponsesOutput>,
}

impl ResponseResponse {
    pub fn content(&self) -> Response {
        let reasoning = self
            .output
            .iter()
            .filter(|o| o.r#type == "reasoning")
            .map(|o| o.content())
            .collect::<Vec<String>>()
            .join("\n");

        let content = self
            .output
            .iter()
            .filter(|o| o.r#type == "message")
            .map(|o| o.content())
            .collect::<Vec<String>>()
            .join("\n");

        Response {
            content,
            reasoning: if reasoning.is_empty() {
                None
            } else {
                Some(reasoning)
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesOutput {
    pub r#type: String,
    pub content: Vec<ResponsesContent>,
}

impl ResponsesOutput {
    fn content(&self) -> String {
        self.content
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesContent {
    pub r#type: String,
    pub text: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Provider {
    pub sort: Option<String>,
    pub order: Option<Vec<String>>,
}

impl CompletionRequest {
    pub fn new_translate_request(req: TranslateRequest) -> Self {
        let mut msgs = vec![Message {
            role: "system".to_string(),
            content: system_msg(req.chars_limit).to_string(),
            ..Default::default()
        }];

        msgs.push(Message {
            role: "user".to_string(),
            content: format!(
                "{}\n{}",
                req.instruction.unwrap_or_default(),
                req.query.to_string()
            ),
            ..Default::default()
        });

        Self {
            model: req.model.to_string(),
            messages: msgs,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

impl Message {
    fn to_response(&self) -> Response {
        Response {
            content: self.content.clone(),
            reasoning: self.reasoning.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct OpenAI {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub reasoning: Option<Reasoning>,
    pub provider: Option<Vec<String>>,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
}

#[derive(Debug, Default)]
pub struct TranslateRequest<'a> {
    pub model: &'a str,
    pub query: &'a str,
    pub chars_limit: bool,
    pub instruction: Option<&'a str>,
    pub dst_lang: Option<&'a str>,
}

impl TranslateRequest<'_> {
    pub fn completion_request(self) -> CompletionRequest {
        CompletionRequest::new_translate_request(self)
    }

    pub fn responses_request(self) -> ResponsesRequest {
        ResponsesRequest::new_translate_request(self)
    }
}

impl OpenAI {
    pub fn from_env(env: String) -> HashMap<String, OpenAI> {
        let env = match base64::engine::general_purpose::STANDARD.decode(env) {
            Ok(v) => v,
            Err(_) => return HashMap::new(),
        };

        match serde_json::from_slice(&env) {
            Ok(v) => v,
            Err(_) => HashMap::new(),
        }
    }

    pub fn new(name: &str, base_url: &str, api_key: &str, models: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            models: HashSet::from_iter(models),
            ..Default::default()
        }
    }

    pub fn set_allow_all_models(&mut self, allow_all_models: bool) -> Self {
        self.allow_all_models = Some(allow_all_models);
        self.clone()
    }

    pub fn enabled(&self) -> bool {
        !self.base_url.is_empty() && !self.models.is_empty()
    }

    pub fn models(&self) -> Vec<String> {
        self.models.iter().cloned().collect()
    }

    pub fn authorization_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    pub async fn exec<I: Serialize, O: DeserializeOwned>(
        &self,
        path: &str,
        input: I,
    ) -> Result<O, Error> {
        let body = serde_json::to_string(&input).unwrap();

        let r = reqwest::Client::builder()
            .build()?
            .post(format!("{}{}", self.base_url, path))
            .header("Authorization", self.authorization_header())
            .header(
                "HTTP-Referer",
                "https://github.com/Asutorufa/hujiang_dictionary",
            )
            .header("X-Title", "hj-dict")
            .body(body)
            .send()
            .await?;

        let status = r.status();
        let text = r.text().await?;

        info!("openai raw result: {}, status: {}", text, status);

        if !status.is_success() {
            return Err(Error(text));
        }

        Ok(serde_json::from_str(&text)?)
    }

    fn allow_model(&self, model: &str) -> Result<(), Error> {
        if !self.allow_all_models.unwrap_or(false) && !self.models.contains(model) {
            return Err(Error("model not supported".to_string()));
        } else {
            Ok(())
        }
    }

    pub async fn completion(
        &self,
        mut req: CompletionRequest,
    ) -> Result<CompletionResponse, Error> {
        self.allow_model(&req.model.to_string())?;

        if self.provider.is_some() {
            req.provider = Some(Provider {
                sort: None,
                order: self.provider.clone(),
            });
        }

        if self.reasoning.is_some() {
            req.reasoning = self.reasoning.clone();
        }

        let r: CompletionResponse = self.exec("/chat/completions", req).await?;
        Ok(r)
    }

    pub async fn responses(&self, mut req: ResponsesRequest) -> Result<ResponseResponse, Error> {
        self.allow_model(&req.model.to_string())?;

        if self.reasoning.is_some() {
            req.reasoning = self.reasoning.clone();
        }

        self.exec("/responses", req).await
    }

    pub fn translate<'a>(
        &self,
        req: TranslateRequest<'a>,
    ) -> impl Future<Output = Result<Response, Error>> {
        async move {
            let instruction = match req.instruction {
                Some(i) if !i.is_empty() => Some(i.to_string()),
                _ => {
                    if req.dst_lang.is_some() {
                        Some(format!("\nTarget Language: {}", req.dst_lang.unwrap()))
                    } else {
                        None
                    }
                }
            };

            let req = TranslateRequest {
                model: req.model,
                query: req.query,
                chars_limit: req.chars_limit,
                instruction: instruction.as_deref(),
                dst_lang: req.dst_lang,
            };

            let result = self.completion(req.completion_request()).await?;

            match result.choices.len() {
                0 => Err(Error("no choice".to_string())),
                _ => Ok(result.choices.first().unwrap().message.to_response()),
            }
        }
    }

    pub fn google_search<'a>(
        &self,
        req: TranslateRequest<'a>,
    ) -> impl Future<Output = Result<Response, Error>> {
        async move {
            let mut instruction = google_search(req.query).await?;

            if req.dst_lang.is_some() {
                instruction.push_str(&format!("\nTarget Language: {}", req.dst_lang.unwrap()));
            }

            info!("google search instruct: {}", instruction);

            let req = TranslateRequest {
                model: req.model,
                query: req.query,
                chars_limit: req.chars_limit,
                instruction: Some(&instruction),
                dst_lang: req.dst_lang,
            };

            let result = self.completion(req.completion_request()).await?;

            match result.choices.len() {
                0 => Err(Error("no choice".to_string())),
                _ => Ok(result.choices.first().unwrap().message.to_response()),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::ai::OpenAI;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .format_line_number(true)
            .init();
    }

    #[tokio::test]
    async fn test() {
        init();

        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let oa = serde_json::from_str::<OpenAI>(&auth_json).unwrap();

        println!("{:?}", oa.models());

        println!(
            "{:?}",
            oa.google_search(crate::ai::TranslateRequest {
                model: oa.models.iter().next().unwrap(),
                query: "日をおかずに 意味",
                dst_lang: Some("ja"),
                ..Default::default()
            })
            .await
        );
    }
}
