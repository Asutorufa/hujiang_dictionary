use std::collections::{HashMap, HashSet};

use base64::Engine;
use hjdict::google_search;
use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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
            match model {
                Models::Gemma3_12bIt | Models::Llama4Scout17B16EInstruct => {
                    let result = self
                        .completion(CompletionRequest::new_workers_ai(
                            model,
                            system_msg(chars_limit),
                            prompt,
                            instruction,
                        ))
                        .await?;

                    match result.choices.len() {
                        0 => Err(Error("no choice".to_string())),
                        _ => Ok(result.choices.first().unwrap().message.to_response()),
                    }
                }

                Models::GPTOss20B => {
                    let result = self
                        .responses(ResponsesRequest::new(
                            model,
                            system_msg(chars_limit),
                            prompt,
                            instruction,
                        ))
                        .await?
                        .content();

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
            let system_msg = system_msg(chars_limit);

            info!("google search instruct: {}", instruction);

            match model {
                Models::Gemma3_12bIt | Models::Llama4Scout17B16EInstruct => {
                    let result = self
                        .completion(CompletionRequest::new_workers_ai(
                            model,
                            system_msg,
                            query,
                            Some(instruction.as_str()),
                        ))
                        .await?;

                    match result.choices.len() {
                        0 => Err(Error("no choice".to_string())),
                        _ => Ok(result.choices.first().unwrap().message.to_response()),
                    }
                }

                Models::GPTOss20B | _ => {
                    let result = self
                        .responses(ResponsesRequest::new(
                            model,
                            system_msg,
                            query,
                            Some(instruction.as_str()),
                        ))
                        .await?
                        .content();

                    Ok(result)
                }
            }
        }
    }
}

async fn google_search(query: &str) -> Result<String, Error> {
    let result = match google_search::get(query).await {
        Ok(v) => v,
        Err(e) => return Err(Error::from(e.to_string())),
    };

    let mut instruct = "<google search result>\n".to_string();

    let length = if result.len() > 3 { 3 } else { result.len() };

    for v in &result[0..length] {
        let title = v.title.replace("\n", " ");
        instruct.push_str("<>\n");
        instruct.push_str(&format!("<title>{}</title>\n", title));
        instruct.push_str(&format!("<link>{}</link>\n", v.url));
        instruct.push_str(&format!(
            "<content>{}</content>\n",
            v.get_raw_page().await.unwrap()
        ));
        instruct.push_str("</>\n");
    }

    instruct.push_str("\n</google search result>\n");

    instruct.push_str("\n**please use above google search result to explain.**\n");

    Ok(instruct)
}

pub static SYSTEM_MSG: &str = r#"
You are a professional translator.
Translate the input text according to the user's instructions and return the result in the user’s original language (unless the user requests otherwise).
"#;

pub static SYSTEM_MSG_LIMIT: &str = r#"
You are a professional translator.
Translate the input text according to the user's instructions and return the result in the user’s original language (unless the user requests otherwise).
If the translated content is approaching the limit, prioritize preserving core meaning and compress the expression when necessary. Paraphrase or summarize if required.
Do not output in Markdown format.
The total output must not exceed 4096 characters, including spaces and line breaks.
Strictly follow the character limit to prevent truncation.
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

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesRequest {
    pub instructions: String,
    pub model: String,
    pub input: Vec<Message>,
    pub reasoning: Reasoning,
}

impl ResponsesRequest {
    pub fn new(model: Models, system: &str, prompt: &str, instruction: Option<&str>) -> Self {
        let mut msgs = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
            reasoning: None,
        }];

        if let Some(instruction) = instruction {
            msgs.push(Message {
                role: "system".to_string(),
                content: instruction.to_string(),
                reasoning: None,
            });
        }

        Self {
            instructions: system.to_string(),
            model: model.as_str().to_string(),
            input: msgs,
            reasoning: Reasoning {
                effort: "low".to_string(),
                summary: "concise".to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reasoning {
    pub effort: String,
    pub summary: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub reasoning: Option<Reasoning>,
}

impl CompletionRequest {
    pub fn new(model: &str, system: &str, prompt: &str, instruction: Option<&str>) -> Self {
        let mut msgs = vec![
            Message {
                role: "system".to_string(),
                content: system.to_string(),
                reasoning: None,
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
                reasoning: None,
            },
        ];

        if let Some(instruction) = instruction {
            msgs.push(Message {
                role: "system".to_string(),
                content: instruction.to_string(),
                reasoning: None,
            });
        }

        Self {
            model: model.to_string(),
            messages: msgs,
            reasoning: None,
        }
    }

    pub fn new_workers_ai(
        model: Models,
        system: &str,
        prompt: &str,
        instruction: Option<&str>,
    ) -> Self {
        CompletionRequest::new(model.as_str(), system, prompt, instruction)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub target_lang: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_lang: Option<String>,
}

impl TranslateRequest {
    pub fn new(text: &str, target_lang: &str, source_lang: Option<String>) -> Self {
        Self {
            text: text.to_string(),
            target_lang: target_lang.to_string(),
            source_lang,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateOutput {
    pub translated_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResult {
    pub result: TranslateOutput,
}

#[derive(serde::Deserialize, Clone)]
pub struct OpenAI {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub models: HashSet<String>,
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
        }
    }

    pub fn enabled(&self) -> bool {
        !self.base_url.is_empty() && !self.models.is_empty()
    }

    pub fn models(&self) -> Vec<String> {
        self.models.iter().cloned().collect()
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header(
                "HTTP-Referer",
                "https://github.com/Asutorufa/hujiang_dictionary",
            )
            .header("X-Title", "hj-dict")
            .body(body)
            .send()
            .await?;

        if r.status() != 200 {
            return Err(Error(r.text().await?));
        }

        Ok(r.json::<O>().await?)
    }

    pub async fn completion(
        &self,
        model: &str,
        system: &str,
        prompt: &str,
        instruction: Option<&str>,
    ) -> Result<CompletionResponse, Error> {
        if !self.models.contains(&model.to_string()) {
            return Err(Error("model not supported".to_string()));
        }

        let mut req = CompletionRequest::new(model, system, prompt, instruction);
        req.reasoning = Some(Reasoning {
            effort: "low".to_string(),
            summary: "concise".to_string(),
        });

        let r: CompletionResponse = self.exec("/chat/completions", req).await?;
        Ok(r)
    }

    pub fn translate(
        &self,
        model: &str,
        chars_limit: bool,
        prompt: &str,
        instruction: Option<&str>,
    ) -> impl Future<Output = Result<Response, Error>> {
        async move {
            let result = self
                .completion(model, system_msg(chars_limit), prompt, instruction)
                .await?;

            match result.choices.len() {
                0 => Err(Error("no choice".to_string())),
                _ => Ok(result.choices.first().unwrap().message.to_response()),
            }
        }
    }

    pub fn google_search(
        &self,
        model: &str,
        chars_limit: bool,
        query: &str,
    ) -> impl Future<Output = Result<Response, Error>> {
        async move {
            let instruction = google_search(query).await?;
            let system_msg = system_msg(chars_limit);

            info!("google search instruct: {}", instruction);

            let result = self
                .completion(model, system_msg, query, Some(instruction.as_str()))
                .await?;

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

    #[tokio::test]
    async fn test() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let oa = serde_json::from_str::<OpenAI>(&auth_json).unwrap();

        println!("{:?}", oa.models());

        println!(
            "{:?}",
            oa.google_search(
                oa.models.iter().next().unwrap(),
                false,
                "辿るは何の意味ですか？",
            )
            .await
        );
    }
}
