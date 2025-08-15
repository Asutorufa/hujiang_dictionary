pub trait AI {
    fn gemma3_12b(&self, prompt: String) -> impl Future<Output = Result<String, Error>>;
    fn llama4_scout_17b_16e_instruct(
        &self,
        prompt: String,
    ) -> impl Future<Output = Result<String, Error>>;
    fn m2m100_1_2b(
        &self,
        text: String,
        source_lang: Option<String>,
        target_lang: String,
    ) -> impl Future<Output = Result<String, Error>>;
}

pub struct EmptyAI {}

impl AI for EmptyAI {
    async fn gemma3_12b(&self, _: String) -> Result<String, Error> {
        Err(Error("empty ai".to_string()))
    }
    async fn llama4_scout_17b_16e_instruct(&self, _: String) -> Result<String, Error> {
        Err(Error("empty ai".to_string()))
    }
    async fn m2m100_1_2b(&self, _: String, _: Option<String>, _: String) -> Result<String, Error> {
        Err(Error("empty ai".to_string()))
    }
}

pub static SYSTEM_MSG: &str = r#"
You are a professional translator.
Translate the input text according to the user's instructions and return the result in the user’s original language (unless the user requests otherwise).
The total output must not exceed 4096 characters, including spaces and line breaks.
If the translated content is approaching the limit, prioritize preserving core meaning and compress the expression when necessary. Paraphrase or summarize if required.
Do not output in Markdown format.
Strictly follow the character limit to prevent truncation.
"#;

pub enum Models {
    Gemma3_12bIt,
    Llama4Scout17B16EInstruct,
    M2M100_1_2B,
}

impl Models {
    pub fn as_str(&self) -> &'static str {
        match self {
            Models::Gemma3_12bIt => "@cf/google/gemma-3-12b-it",
            Models::Llama4Scout17B16EInstruct => "@cf/meta/llama-4-scout-17b-16e-instruct",
            Models::M2M100_1_2B => "@cf/meta/m2m100-1.2b",
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
