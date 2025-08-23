use serde::Deserialize;

#[derive(Debug)]
pub struct Output {
    pub source: String,
    pub translation: String,
}

impl Clone for Output {
    fn clone(&self) -> Self {
        Output {
            source: self.source.clone(),
            translation: self.translation.clone(),
        }
    }
}

pub fn merge_output(outputs: Vec<Output>) -> (String, String) {
    let mut merged_source = String::new();
    let mut merged_translation = String::new();

    for output in outputs {
        merged_source.push_str(&output.source);
        merged_translation.push_str(&output.translation);
    }

    (merged_source, merged_translation)
}

pub fn merge_source(outputs: Vec<Output>) -> String {
    let mut merged_source = String::new();

    for output in outputs {
        merged_source.push_str(&output.source);
    }

    merged_source
}

pub fn merge_translation(outputs: Vec<Output>) -> String {
    let mut merged_translation = String::new();

    for output in outputs {
        merged_translation.push_str(&output.translation);
    }

    merged_translation
}

#[derive(Debug, Deserialize)]
pub struct TranslateResponse {
    /*
    {
      "translation": "Rust は驚くほど高速でメモリ効率に優れています。ランタイムやガベージ コレクターがないため、パフォーマンスが重要なサービスに動力を与え、組み込みデバイスで実行し、他の言語と簡単に統合できます。Rust の豊富な型システムと所有権モデルは、メモリ安全性とスレッド安全性を保証し、コンパイル時に多くの種類のバグを排除できます。Rust には、優れたドキュメント、役立つエラー メッセージを備えた使いやすいコンパイラ、統合されたパッケージ マネージャーとビルド ツール、自動補完と型検査を備えたスマートなマルチエディター サポート、自動フォーマッタなど、一流のツールが備わっています。",
      "sentences": [
        {
          "trans": "Rust は驚くほど高速でメモリ効率に優れています。ランタイムやガベージ コレクターがないため、パフォーマンスが重要なサービスに動力を与え、組み込みデバイスで実行し、他の言語と簡単に統合できます。Rust の豊富な型システムと所有権モデルは、メモリ安全性とスレッド安全性を保証し、コンパイル時に多くの種類のバグを排除できます。Rust には、優れたドキュメント、役立つエラー メッセージを備えた使いやすいコンパイラ、統合されたパッケージ マネージャーとビルド ツール、自動補完と型検査を備えたスマートなマルチエディター サポート、自動フォーマッタなど、一流のツールが備わっています。",
          "orig": "Rust is blazingly fast and memory-efficient: with no runtime or garbage collector, it can power performance-critical services, run on embedded devices, and easily integrate with other languages.Rust’s rich type system and ownership model guarantee memory-safety and thread-safety — enabling you to eliminate many classes of bugs at compile-time.Rust has great documentation, a friendly compiler with useful error messages, and top-notch tooling — an integrated package manager and build tool, smart multi-editor support with auto-completion and type inspections, an auto-formatter, and more."
        }
      ],
      "detectedLanguages": {
        "srclangs": [
          "en"
        ],
        "extendedSrclangs": [
          "en"
        ]
      },
      "sourceLanguage": "en"
    }
    */
    pub translation: String,
    pub sentences: Vec<Sentence>,
    #[serde(rename = "detectedLanguages")]
    pub detected_languages: DetectedLanguages,
    #[serde(rename = "sourceLanguage")]
    pub source_language: String,
}

#[derive(Debug, Deserialize)]
pub struct Sentence {
    pub trans: String,
    pub orig: String,
}

#[derive(Debug, Deserialize)]
pub struct DetectedLanguages {
    pub srclangs: Vec<String>,
    #[serde(rename = "extendedSrclangs")]
    pub extended_srclangs: Vec<String>,
}

pub async fn translatev2(
    text: &str,
    src: Option<String>,
    target: &str,
) -> Result<Vec<Output>, reqwest::Error> {
    /*

    translate-pa.googleapis.com
    :method
    GET
    :path
    /v1/translate?
        params.client=gtx
        &
        query.source_language=ja
        &
        query.target_language=zh-CN
        &
        query.display_language=ja
        &
        query.text=%E3%83%91%E3%83%95%E3%82%A9%E3%83%BC%E3%83%9E%E3%83%B3%E3%82%B9
        &
        key=AIzaSyDLEeFI5OtFBwYBIoK_jj5m32rZK5CkCXA
        &
        data_types=TRANSLATION
        &
        data_types=SENTENCE_SPLITS
        &
        data_types=BILINGUAL_DICTIONARY_FULL
        */
    let resp = reqwest::Client::builder()
        .build()?
        .get("https://translate-pa.googleapis.com/v1/translate")
        .query(&vec![
            ("params.client".to_string(), "gtx".to_string()),
            (
                "query.source_language".to_string(),
                src.unwrap_or("auto".to_string()),
            ),
            ("query.target_language".to_string(), target.to_string()),
            ("query.display_language".to_string(), target.to_string()),
            ("query.text".to_string(), text.to_string()),
            (
                "key".to_string(),
                "AIzaSyDLEeFI5OtFBwYBIoK_jj5m32rZK5CkCXA".to_string(),
            ),
            ("data_types".to_string(), "TRANSLATION".to_string()),
            ("data_types".to_string(), "SENTENCE_SPLITS".to_string()),
            (
                "data_types".to_string(),
                "BILINGUAL_DICTIONARY_FULL".to_string(),
            ),
        ])
        .header(
            "User-Agent".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36".to_string(),
        )
        // .header("Content-Type".to_string(), "application/json+protobuf".to_string())
        .send()
        .await?;

    if resp.status() != 200 {
        let text = resp.text().await?;
        return Ok(vec![Output {
            source: text.clone(),
            translation: text,
        }]);
    }

    let body = resp.json::<TranslateResponse>().await?;

    println!("{:?}", body);

    let outputs = body
        .sentences
        .iter()
        .map(|v| Output {
            source: v.orig.clone(),
            translation: v.trans.clone(),
        })
        .collect::<Vec<_>>();

    Ok(outputs)
}

pub async fn translate(
    text: &str,
    src: Option<String>,
    target: &str,
) -> Result<Vec<Output>, reqwest::Error> {
    let resp = reqwest::Client::builder()
        .build()?
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&vec![
            ("client".to_string(), "gtx".to_string()),
            ("dt".to_string(), "at".to_string()),
            ("dt".to_string(), "bd".to_string()),
            ("dt".to_string(), "ex".to_string()),
            ("dt".to_string(), "ld".to_string()),
            ("dt".to_string(), "md".to_string()),
            ("dt".to_string(), "qca".to_string()),
            ("dt".to_string(), "rw".to_string()),
            ("dt".to_string(), "rm".to_string()),
            ("dt".to_string(), "ss".to_string()),
            ("dt".to_string(), "t".to_string()),
            ("kc".to_string(), "7".to_string()),
            ("otf".to_string(), "1".to_string()),
            ("ssel".to_string(), "0".to_string()),
            ("tsel".to_string(), "0".to_string()),
            ("ie".to_string(), "UTF-8".to_string()),
            ("oe".to_string(), "UTF-8".to_string()),
            ("q".to_string(), text.to_string()),
            (
                "sl".to_string(),
                match src {
                    Some(v) if !v.is_empty() => v.to_string(),
                    _ => "auto".to_string(),
                },
            ),
            ("tl".to_string(), target.to_string()),
            ("hl".to_string(), target.to_string()),
        ])
        .send()
        .await?;

    if resp.status() != 200 {
        let text = resp.text().await?;
        return Ok(vec![Output {
            source: text.clone(),
            translation: text,
        }]);
    }

    let body: Vec<serde_json::Value> = resp.json().await?;

    let mut outputs = vec![];

    if body.len() < 1 || !body[0].is_array() {
        return Ok(outputs);
    }

    for b in body[0].as_array().unwrap() {
        if !b.is_array() {
            continue;
        }

        let ba = b.as_array().unwrap();
        if ba.len() < 2 {
            continue;
        }

        if !ba[0].is_string() || !ba[1].is_string() {
            continue;
        }

        let dst = ba[0].as_str().unwrap();
        let src = ba[1].as_str().unwrap();

        outputs.push(Output {
            source: src.to_string(),
            translation: dst.to_string(),
        });
    }

    Ok(outputs)
}

#[cfg(test)]
mod test {
    use crate::google::{merge_output, merge_source, merge_translation};

    #[tokio::test]
    async fn test_translate() {
        let text = "Rust is blazingly fast and memory-efficient: with no runtime or garbage collector, it can power performance-critical services, run on embedded devices, and easily integrate with other languages.Rust’s rich type system and ownership model guarantee memory-safety and thread-safety — enabling you to eliminate many classes of bugs at compile-time.Rust has great documentation, a friendly compiler with useful error messages, and top-notch tooling — an integrated package manager and build tool, smart multi-editor support with auto-completion and type inspections, an auto-formatter, and more.";
        let target = "ja";

        let out = super::translatev2(text, None, target).await.unwrap();

        println!("{:?}", merge_source(out.clone()));
        println!("{:?}", merge_translation(out.clone()));
        println!("{:?}", merge_output(out));
    }
}
