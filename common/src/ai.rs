use serde::{Deserialize, Serialize};

pub trait AI {
    fn gemma3_12b(&self, prompt: &str) -> impl Future<Output = Result<String, Error>>;
    fn llama4_scout_17b_16e_instruct(
        &self,
        prompt: &str,
    ) -> impl Future<Output = Result<String, Error>>;
    fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> impl Future<Output = Result<String, Error>>;
    fn gpt_oss_20b(&self, prompt: &str) -> impl Future<Output = Result<String, Error>>;
}

pub static SYSTEM_MSG: &str = r#"
You are a professional translator.
Translate the input text according to the user's instructions and return the result in the user’s original language (unless the user requests otherwise).
The total output must not exceed 4096 characters, including spaces and line breaks.
If the translated content is approaching the limit, prioritize preserving core meaning and compress the expression when necessary. Paraphrase or summarize if required.
Do not output in Markdown format.
Strictly follow the character limit to prevent truncation.
"#;

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
    pub input: String,
    pub reasoning: Reasoning,
}

impl ResponsesRequest {
    pub fn new(model: Models, prompt: &str) -> Self {
        Self {
            instructions: SYSTEM_MSG.to_string(),
            model: model.as_str().to_string(),
            input: prompt.to_string(),
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
    pub fn content(&self) -> Vec<(String, String)> {
        self.output
            .iter()
            .map(|o| {
                o.content
                    .iter()
                    .map(|c| (c.r#type.clone(), c.text.clone()))
                    .collect()
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsesOutput {
    pub content: Vec<ResponsesContent>,
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
}

impl CompletionRequest {
    pub fn new(model: Models, prompt: &str) -> Self {
        Self {
            model: model.as_str().to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: SYSTEM_MSG.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
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
