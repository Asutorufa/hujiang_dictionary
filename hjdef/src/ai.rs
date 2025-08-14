pub trait AI: Send + Sync + Clone + 'static {
    fn gemma3_12b(&self, prompt: String) -> impl Future<Output = Result<String, Error>> + Send;
    fn llama4_scout_17b_16e_instruct(
        &self,
        prompt: String,
    ) -> impl Future<Output = Result<String, Error>> + Send;
    fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> impl Future<Output = Result<String, Error>> + Send;
}

#[derive(Debug, Clone)]
pub struct Error(String);

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
