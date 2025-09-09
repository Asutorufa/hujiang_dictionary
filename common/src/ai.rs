use hjdict::google_search;
use log::info;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub trait AI {
    fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> impl Future<Output = Result<String, Error>>;

    fn gemma3_12b(
        &self,
        system: &str,
        prompt: &str,
        instruction: Option<&str>,
    ) -> impl Future<Output = Result<String, Error>>;
    fn llama4_scout_17b_16e_instruct(
        &self,
        system: &str,
        prompt: &str,
        instruction: Option<&str>,
    ) -> impl Future<Output = Result<String, Error>>;
    fn gpt_oss_20b(
        &self,
        system: &str,
        prompt: &str,
        instruction: Option<&str>,
    ) -> impl Future<Output = Result<String, Error>>;

    fn translate(
        &self,
        model: Models,
        chars_limit: bool,
        prompt: &str,
        instruction: Option<&str>,
    ) -> impl Future<Output = Result<String, Error>> {
        async move {
            match model {
                Models::Gemma3_12bIt => {
                    self.gemma3_12b(system_msg(chars_limit), prompt, instruction)
                        .await
                }
                Models::Llama4Scout17B16EInstruct => {
                    self.llama4_scout_17b_16e_instruct(system_msg(chars_limit), prompt, instruction)
                        .await
                }
                Models::GPTOss20B => {
                    self.gpt_oss_20b(system_msg(chars_limit), prompt, instruction)
                        .await
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
    ) -> impl Future<Output = Result<String, Error>> {
        async move {
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

            info!("google search instruct: {}", instruct);

            match model {
                Models::GPTOss20B => {
                    self.gpt_oss_20b(system_msg(chars_limit), query, Some(instruct.as_str()))
                        .await
                }
                Models::Gemma3_12bIt => {
                    self.gemma3_12b(system_msg(chars_limit), query, Some(instruct.as_str()))
                        .await
                }
                Models::Llama4Scout17B16EInstruct => {
                    self.llama4_scout_17b_16e_instruct(
                        system_msg(chars_limit),
                        query,
                        Some(instruct.as_str()),
                    )
                    .await
                }
                _ => {
                    self.gpt_oss_20b(system_msg(chars_limit), query, Some(instruct.as_str()))
                        .await
                }
            }
        }
    }
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
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl OpenAI {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
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
        system: &str,
        prompt: &str,
        instruction: Option<&str>,
    ) -> Result<Message, Error> {
        let mut req = CompletionRequest::new(self.model.as_str(), system, prompt, instruction);
        req.reasoning = Some(Reasoning {
            effort: "low".to_string(),
            summary: "concise".to_string(),
        });

        let r: CompletionResponse = self.exec("/chat/completions", req).await?;
        Ok(r.choices
            .first()
            .ok_or(Error("choice is empty".to_string()))?
            .message
            .clone())
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::ai::{OpenAI, system_msg};

    #[tokio::test]
    async fn test() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let oa = serde_json::from_str::<OpenAI>(&auth_json).unwrap();

        println!(
            "{:?}",
            oa.completion(system_msg(false), "辿るは何の意味ですか？", None)
                .await
                .unwrap()
        );
    }
}
