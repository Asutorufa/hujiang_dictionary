use crate::{
    en::{COOKIE, USER_AGENT},
    error::Error,
};
use log::info;
use scraper::Html;
use std::fmt::Write;
use url::form_urlencoded;

#[derive(Debug, Default)]
pub struct Word {
    word: String,
    katakana: String,
    audio_url: String,
    simple: Vec<SimpleExplain>,
    detail: Vec<Detail>,
}

#[derive(Debug, Default)]
pub struct SimpleExplain {
    attribute: String,
    explains: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Detail {
    attribute: String,
    source: String,
    explains_and_example: Vec<ExplainsAndExample>,
}

#[derive(Debug)]
pub struct Example {
    pub original: String,
    pub translate: String,
}

#[derive(Debug, Default)]
pub struct ExplainsAndExample {
    explain: String,
    example: Vec<Example>,
}

impl Word {
    pub fn markdown(&self) -> String {
        let mut s = String::new();

        // Add word, katakana, and audio
        writeln!(s, "{}", self.word).unwrap();
        writeln!(s, "{}", self.katakana).unwrap();
        write!(
            s,
            r#"<audio controls controlsList="nodownload" preload="none" src="{}"></audio>"#,
            self.audio_url
        )
        .unwrap();
        s.push('\n');

        // Handle simple explanations
        for (i2, simple) in self.simple.iter().enumerate() {
            if i2 == 0 {
                s.push_str("\n- simple explain\n");
            }

            if !simple.attribute.is_empty() {
                writeln!(s, "  - {}  ", simple.attribute).unwrap();
            } else {
                s.push_str("  - *\n");
            }

            for explain in &simple.explains {
                writeln!(s, "    - {}  ", explain).unwrap();
            }
        }

        // Handle detailed explanations
        for (i2, detail) in self.detail.iter().enumerate() {
            if i2 == 0 {
                s.push_str("\n- More Detail\n");
            }

            writeln!(s, "  - {}  ", detail.attribute).unwrap();

            for example in &detail.explains_and_example {
                writeln!(s, "    - {}  ", example.explain).unwrap();

                for e in &example.example {
                    writeln!(s, "      - {}  ", e.original).unwrap();
                    writeln!(s, "        {}  ", e.translate).unwrap();
                }
            }
        }

        s
    }
}

pub async fn get(word: &str) -> Result<Vec<Word>, Error> {
    info!("Fetching Korean dictionary for word: {}", word);

    let r = reqwest::Client::builder()
        .build()?
        .get(format!(
            "https://dict.hjenglish.com/kr/{}",
            form_urlencoded::byte_serialize(word.as_bytes()).collect::<String>()
        ))
        .header("User-Agent", USER_AGENT)
        .header("Cookie", COOKIE)
        .send()
        .await?;

    let status = r.status();

    if status != 200 {
        return Err(Error {
            message: r.text().await?,
            status: Some(status),
        });
    }

    let text = r.text().await?;

    // println!("{}", text);
    Ok(parse(&text))
}

fn parse(html: &str) -> Vec<Word> {
    let document = Html::parse_document(html);

    let mut words = Vec::new();

    for pane in document.select(sel!(".word-details-pane")) {
        let mut word = Word {
            word: pane
                .select(sel!(".word-text h2"))
                .next()
                .map(|n| n.text().collect::<String>())
                .unwrap_or_default(),
            ..Default::default()
        };

        if let Some(p) = pane.select(sel!(".pronounces")).next() {
            word.katakana = p
                .select(sel!("span"))
                .next()
                .map(|n| n.text().collect())
                .unwrap_or_default();

            word.audio_url = p
                .select(sel!(".word-audio"))
                .next()
                .and_then(|n| n.value().attr("data-src"))
                .unwrap_or("")
                .to_string();
        }

        for simple in pane.select(sel!(".simple")) {
            let attrs: Vec<_> = simple.select(sel!("h2")).collect();

            if attrs.is_empty() {
                let text = re_sum(&simple.text().collect::<String>());
                if !text.is_empty() {
                    word.simple.push(SimpleExplain {
                        attribute: String::new(),
                        explains: vec![text],
                    });
                }
                continue;
            }

            let lists: Vec<_> = simple.select(sel!("ul")).collect();

            for (i, attr) in attrs.iter().enumerate() {
                let mut se = SimpleExplain {
                    attribute: attr.text().collect(),
                    ..Default::default()
                };

                if let Some(ul) = lists.get(i) {
                    for li in ul.select(sel!("li")) {
                        se.explains.push(li.text().collect());
                    }
                }

                word.simple.push(se);
            }
        }

        word.detail = get_details(&pane);

        words.push(word);
    }

    words
}

fn get_details(pane: &scraper::ElementRef) -> Vec<Detail> {
    let mut details = Vec::new();

    for item in pane.select(sel!(".word-details-pane-content .word-details-item")) {
        let source: String = item
            .select(sel!(".detail-source"))
            .next()
            .map(|n| n.text().collect())
            .unwrap_or_default();

        for dl in item.select(sel!(".detail-groups dl")) {
            let mut detail = get_detail(&dl);
            if detail.explains_and_example.is_empty() {
                continue;
            }
            detail.source = source.clone();
            details.push(detail);
        }
    }

    details
}

fn get_detail(dl: &scraper::ElementRef) -> Detail {
    let mut detail = Detail {
        attribute: dl
            .select(sel!("dt"))
            .next()
            .map(|n| re_sum(&n.text().collect::<String>()))
            .unwrap_or_default(),
        ..Default::default()
    };

    for dd in dl.select(sel!("dd")) {
        let mut explain = String::new();
        for h3 in dd.select(sel!("h3")) {
            explain.push_str(&re_sum(&h3.text().collect::<String>()));
        }

        let mut eae = ExplainsAndExample {
            explain,
            example: Vec::new(),
        };

        for li in dd.select(sel!("ul li")) {
            let from = li
                .select(sel!(".def-sentence-from"))
                .next()
                .map(|n| re_sum(&n.text().collect::<String>()))
                .unwrap_or_default();

            let to = li
                .select(sel!(".def-sentence-to"))
                .next()
                .map(|n| re_sum(&n.text().collect::<String>()))
                .unwrap_or_default();

            eae.example.push(Example {
                original: from,
                translate: to,
            });
        }

        detail.explains_and_example.push(eae);
    }

    detail
}

fn re_sum(s: &str) -> String {
    s.trim().replace(['\n', ' '], "")
}

#[cfg(test)]
mod tes {
    use crate::kr::{get, parse};
    use std::fs;

    #[tokio::test]
    async fn run_parse() {
        let test1 = fs::read_to_string("../assets/test_data/kr.html.txt").unwrap();
        let test2 = fs::read_to_string("../assets/test_data/kr2.html.txt").unwrap();

        println!("{:?}", parse(test1.as_str()));
        println!("{:?}", parse(test2.as_str()));

        for v in parse(test1.as_str()) {
            println!("{}\n", v.markdown())
        }

        for v in parse(test2.as_str()) {
            println!("{}\n", v.markdown())
        }

        println!(
            "{}",
            get("안녕하세요")
                .await
                .unwrap()
                .iter()
                .map(|v| v.markdown())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
