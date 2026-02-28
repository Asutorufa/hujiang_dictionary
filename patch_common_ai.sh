#!/bin/bash
cat << 'INNER_EOF' > common/src/ai.rs
use std::collections::{HashMap, HashSet};
use std::future::Future;

use base64::Engine;
use log::info;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

#[derive(RustEmbed)]
#[folder = "config/"]
pub struct Assets;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error(value.to_string())
    }
}

pub trait Translator {
    fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> impl Future<Output = Result<String, Error>>;
}

async fn google_search(query: &str) -> Result<String, Error> {
    let mut q = query.to_string();

    match hjdict::duckduckgo_search::get(&query).await {
        Ok(v) => {
            q.push_str("\n\n### DuckDuckGo Search Results:\n");
            for i in v {
                match i {
                    hjdict::google_search::Body::Content(c) => {
                        q.push_str(&format!("{}\n\n", c));
                    }
                    hjdict::google_search::Body::Link(l) => {
                        q.push_str(&format!("#### [{}]({})\n\n", l.title, l.url));
                    }
                }
            }
            Ok(q)
        }
        Err(e) => {
            info!("google search error: {}", e);
            Ok(query.to_string())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Models {
    Gemma3_12bIt,
    Llama4Scout17B16EInstruct,
    DeepSeekR1DistillQwen32b,
    GPTOss20B,
}

impl Models {
    pub fn as_str(&self) -> &str {
        match self {
            Models::Gemma3_12bIt => "@cf/google/gemma-3-12b-it",
            Models::Llama4Scout17B16EInstruct => "@cf/meta/llama-3.1-8b-instruct", // mapping placeholder
            Models::DeepSeekR1DistillQwen32b => "@cf/deepseek-ai/deepseek-r1-distill-qwen-32b",
            Models::GPTOss20B => "gpt-oss-20b",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn from_model_name(model_name: &str) -> Option<Self> {
        match model_name {
            "@cf/google/gemma-3-12b-it" => Some(Models::Gemma3_12bIt),
            "@cf/meta/llama-3.1-8b-instruct" => Some(Models::Llama4Scout17B16EInstruct),
            "@cf/deepseek-ai/deepseek-r1-distill-qwen-32b" => Some(Models::DeepSeekR1DistillQwen32b),
            "gpt-oss-20b" => Some(Models::GPTOss20B),
            _ => None,
        }
    }

    pub fn llms() -> Vec<String> {
        vec![
            Models::Gemma3_12bIt.to_string(),
            Models::Llama4Scout17B16EInstruct.to_string(),
            Models::DeepSeekR1DistillQwen32b.to_string(),
            Models::GPTOss20B.to_string(),
        ]
    }
}

fn system_msg(chars_limit: bool) -> &'static str {
    if chars_limit {
        r#"You are a professional, authentic translation engine, only returns translations.
- For words, phrases, or short sentences, provide the translation directly. Include essential explanations only if the context is ambiguous or the user explicitly requests it.
- If the translated content is approaching the limit, prioritize preserving core meaning and compress the expression when necessary. Paraphrase or summarize if required.
"#
    } else {
        r#"You are a professional, authentic translation engine, only returns translations."#
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Response {
    pub content: String,
    pub reasoning: Option<String>,
}

#[derive(Debug, Default)]
pub struct TranslateRequest<'a> {
    pub model: &'a str,
    pub query: &'a str,
    pub chars_limit: bool,
    pub instruction: Option<&'a str>,
    pub dst_lang: Option<&'a str>,
}

pub async fn explain<'a>(
    provider: &hj_ai::provider::Provider,
    google_search_flag: bool,
    req: TranslateRequest<'a>,
) -> Result<Response, Error> {
    if google_search_flag {
        info!("google search enabled, model: {}", req.model);
        google_search_req(provider, req).await
    } else {
        translate(provider, req).await
    }
}

pub async fn translate<'a>(
    provider: &hj_ai::provider::Provider,
    req: TranslateRequest<'a>,
) -> Result<Response, Error> {
    let instruction = match req.instruction {
        Some(i) if !i.is_empty() => Some(i.to_string()),
        _ => req
            .dst_lang
            .map(|dst| format!("\nTarget Language: {}", dst)),
    };

    let req2 = TranslateRequest {
        model: req.model,
        query: req.query,
        chars_limit: req.chars_limit,
        instruction: instruction.as_deref(),
        dst_lang: req.dst_lang,
    };

    let messages = vec![
        hj_ai::Message {
            role: "system".to_string(),
            content: system_msg(req2.chars_limit).to_string(),
        },
        hj_ai::Message {
            role: "user".to_string(),
            content: format!("{}\n{}", req2.instruction.unwrap_or_default(), req2.query),
        },
    ];

    use hj_ai::Completion;
    let res = provider.completion(messages).await.map_err(|e| Error(e.to_string()))?;

    Ok(Response {
        content: res.content,
        reasoning: res.thinking,
    })
}

pub async fn google_search_req<'a>(
    provider: &hj_ai::provider::Provider,
    req: TranslateRequest<'a>,
) -> Result<Response, Error> {
    let mut instruction = google_search(req.query).await?;

    if let Some(dst) = req.dst_lang {
        instruction.push_str(&format!("\nTarget Language: {}", dst));
    }

    info!("google search instruct: {}", instruction);

    let messages = vec![
        hj_ai::Message {
            role: "system".to_string(),
            content: system_msg(req.chars_limit).to_string(),
        },
        hj_ai::Message {
            role: "user".to_string(),
            content: format!("{}\n{}", instruction, req.query),
        },
    ];

    use hj_ai::Completion;
    let res = provider.completion(messages).await.map_err(|e| Error(e.to_string()))?;

    Ok(Response {
        content: res.content,
        reasoning: res.thinking,
    })
}

pub async fn explain_stream<'a>(
    provider: &hj_ai::provider::Provider,
    google_search_flag: bool,
    req: TranslateRequest<'a>,
) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>, Error> {
    if google_search_flag {
        info!("google search enabled, model: {}", req.model);
        google_search_req_stream(provider, req).await
    } else {
        translate_stream(provider, req).await
    }
}

pub async fn translate_stream<'a>(
    provider: &hj_ai::provider::Provider,
    req: TranslateRequest<'a>,
) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>, Error> {
    let instruction = match req.instruction {
        Some(i) if !i.is_empty() => Some(i.to_string()),
        _ => req
            .dst_lang
            .map(|dst| format!("\nTarget Language: {}", dst)),
    };

    let req2 = TranslateRequest {
        model: req.model,
        query: req.query,
        chars_limit: req.chars_limit,
        instruction: instruction.as_deref(),
        dst_lang: req.dst_lang,
    };

    let messages = vec![
        hj_ai::Message {
            role: "system".to_string(),
            content: system_msg(req2.chars_limit).to_string(),
        },
        hj_ai::Message {
            role: "user".to_string(),
            content: format!("{}\n{}", req2.instruction.unwrap_or_default(), req2.query),
        },
    ];

    use hj_ai::Completion;
    let stream = provider.completion_stream(messages).await.map_err(|e| Error(e.to_string()))?;

    use futures_util::StreamExt;
    let mapped_stream = stream.map(|res| {
        match res {
            Ok(v) => Ok(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&v).unwrap_or_default()))),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    });

    Ok(Box::pin(mapped_stream))
}

pub async fn google_search_req_stream<'a>(
    provider: &hj_ai::provider::Provider,
    req: TranslateRequest<'a>,
) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>, Error> {
    let mut instruction = google_search(req.query).await?;

    if let Some(dst) = req.dst_lang {
        instruction.push_str(&format!("\nTarget Language: {}", dst));
    }

    info!("google search instruct: {}", instruction);

    let messages = vec![
        hj_ai::Message {
            role: "system".to_string(),
            content: system_msg(req.chars_limit).to_string(),
        },
        hj_ai::Message {
            role: "user".to_string(),
            content: format!("{}\n{}", instruction, req.query),
        },
    ];

    use hj_ai::Completion;
    let stream = provider.completion_stream(messages).await.map_err(|e| Error(e.to_string()))?;

    use futures_util::StreamExt;
    let mapped_stream = stream.map(|res| {
        match res {
            Ok(v) => Ok(bytes::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&v).unwrap_or_default()))),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    });

    Ok(Box::pin(mapped_stream))
}

#[derive(serde::Deserialize)]
struct ConfigProvider {
    name: String,
    base_url: String,
    api_key: String,
    models: Vec<String>,
}

pub fn providers_from_assets() -> HashMap<String, hj_ai::provider::Provider> {
    let mut llms = HashMap::new();
    for v in Assets::iter() {
        if !v.ends_with(".json") {
            continue;
        }

        let f = match Assets::get(&v) {
            Some(v) => v,
            None => continue,
        };

        if let Ok(config) = serde_json::from_slice::<HashMap<String, ConfigProvider>>(&f.data) {
            for (k, v) in config {
                llms.insert(
                    k,
                    hj_ai::provider::Provider::OpenAI(hj_ai::openai::OpenAI {
                        name: v.name,
                        base_url: v.base_url,
                        api_key: v.api_key,
                        model: v.models.first().cloned().unwrap_or_default(),
                        models: HashSet::from_iter(v.models),
                        ..Default::default()
                    }),
                );
            }
        }
    }
    llms
}

pub fn providers_from_base64_string(env: String) -> HashMap<String, hj_ai::provider::Provider> {
    let env_bytes = match base64::engine::general_purpose::STANDARD.decode(env) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let config: HashMap<String, ConfigProvider> = match serde_json::from_slice(&env_bytes) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let mut llms = HashMap::new();
    for (k, v) in config {
        llms.insert(
            k,
            hj_ai::provider::Provider::OpenAI(hj_ai::openai::OpenAI {
                name: v.name,
                base_url: v.base_url,
                api_key: v.api_key,
                model: v.models.first().cloned().unwrap_or_default(),
                models: HashSet::from_iter(v.models),
                ..Default::default()
            }),
        );
    }
    llms
}
INNER_EOF
