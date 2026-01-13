use crate::{
    en::{COOKIE, USER_AGENT},
    error::Error,
};
use scraper::{Html, Selector};
use std::fmt::Write;

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
        write!(s, "{}\n", self.word).unwrap();
        write!(s, "{}\n", self.katakana).unwrap();
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
                write!(s, "  - {}  \n", simple.attribute).unwrap();
            } else {
                s.push_str("  - *\n");
            }

            for explain in &simple.explains {
                write!(s, "    - {}  \n", explain).unwrap();
            }
        }

        // Handle detailed explanations
        for (i2, detail) in self.detail.iter().enumerate() {
            if i2 == 0 {
                s.push_str("\n- More Detail\n");
            }

            write!(s, "  - {}  \n", detail.attribute).unwrap();

            for example in &detail.explains_and_example {
                write!(s, "    - {}  \n", example.explain).unwrap();

                for e in &example.example {
                    write!(s, "      - {}  \n", e.original).unwrap();
                    write!(s, "        {}  \n", e.translate).unwrap();
                }
            }
        }

        s
    }
}

pub async fn get(word: &str) -> Result<Vec<Word>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://dict.hjenglish.com/kr/{}", word))
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

    let pane_sel = Selector::parse(".word-details-pane").unwrap();
    let word_sel = Selector::parse(".word-text h2").unwrap();
    let pronounce_sel = Selector::parse(".pronounces").unwrap();
    let span_sel = Selector::parse("span").unwrap();
    let audio_sel = Selector::parse(".word-audio").unwrap();
    let simple_sel = Selector::parse(".simple").unwrap();
    let h2_sel = Selector::parse("h2").unwrap();
    let ul_sel = Selector::parse("ul").unwrap();
    let li_sel = Selector::parse("li").unwrap();

    let mut words = Vec::new();

    for pane in document.select(&pane_sel) {
        let mut word = Word::default();

        word.word = pane
            .select(&word_sel)
            .next()
            .map(|n| n.text().collect::<String>())
            .unwrap_or_default();

        if let Some(p) = pane.select(&pronounce_sel).next() {
            word.katakana = p
                .select(&span_sel)
                .next()
                .map(|n| n.text().collect())
                .unwrap_or_default();

            word.audio_url = p
                .select(&audio_sel)
                .next()
                .and_then(|n| n.value().attr("data-src"))
                .unwrap_or("")
                .to_string();
        }

        for simple in pane.select(&simple_sel) {
            let attrs: Vec<_> = simple.select(&h2_sel).collect();

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

            let lists: Vec<_> = simple.select(&ul_sel).collect();

            for (i, attr) in attrs.iter().enumerate() {
                let mut se = SimpleExplain::default();
                se.attribute = attr.text().collect();

                if let Some(ul) = lists.get(i) {
                    for li in ul.select(&li_sel) {
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
    let item_sel = Selector::parse(".word-details-pane-content .word-details-item").unwrap();
    let source_sel = Selector::parse(".detail-source").unwrap();
    let dl_sel = Selector::parse(".detail-groups dl").unwrap();

    let mut details = Vec::new();

    for item in pane.select(&item_sel) {
        let source: String = item
            .select(&source_sel)
            .next()
            .map(|n| n.text().collect())
            .unwrap_or_default();

        for dl in item.select(&dl_sel) {
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
    let dt_sel = Selector::parse("dt").unwrap();
    let dd_sel = Selector::parse("dd").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let li_sel = Selector::parse("ul li").unwrap();
    let from_sel = Selector::parse(".def-sentence-from").unwrap();
    let to_sel = Selector::parse(".def-sentence-to").unwrap();

    let mut detail = Detail::default();

    detail.attribute = dl
        .select(&dt_sel)
        .next()
        .map(|n| re_sum(&n.text().collect::<String>()))
        .unwrap_or_default();

    for dd in dl.select(&dd_sel) {
        let mut explain = String::new();
        for h3 in dd.select(&h3_sel) {
            explain.push_str(&re_sum(&h3.text().collect::<String>()));
        }

        let mut eae = ExplainsAndExample {
            explain,
            example: Vec::new(),
        };

        for li in dd.select(&li_sel) {
            let from = li
                .select(&from_sel)
                .next()
                .map(|n| re_sum(&n.text().collect::<String>()))
                .unwrap_or_default();

            let to = li
                .select(&to_sel)
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
    s.trim().replace('\n', "").replace(' ', "")
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
