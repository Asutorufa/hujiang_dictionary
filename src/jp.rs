use scraper::{ElementRef, Selector};
use std::fmt::Write;

#[derive(Debug)]
pub struct Simple {
    pub attribute: String,
    pub explains: Vec<String>,
}

#[derive(Debug)]
pub struct Example {
    pub original: String,
    pub translate: String,
}

#[derive(Debug)]
pub struct ExplainsAndExample {
    pub explain: String,
    pub examples: Vec<Example>,
}

#[derive(Debug)]
pub struct Detail {
    pub source: String,
    pub attribute: String,
    pub explains: Vec<ExplainsAndExample>,
}

#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub katakana: String,
    pub audio_url: String,
    pub simple: Vec<Simple>,
    pub detail: Vec<Detail>,
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
                write!(s, "  - {}\n", simple.attribute).unwrap();
            } else {
                s.push_str("  - *\n");
            }

            for explain in &simple.explains {
                write!(s, "    - {}\n", explain).unwrap();
            }
        }

        // Handle detailed explanations
        for (i2, detail) in self.detail.iter().enumerate() {
            if i2 == 0 {
                s.push_str("\n- More Detail\n");
            }

            write!(s, "  - {}\n", detail.attribute).unwrap();

            for example in &detail.explains {
                write!(s, "    - {}\n", example.explain).unwrap();

                for e in &example.examples {
                    write!(s, "      - {}\n", e.original).unwrap();
                    write!(s, "        {}\n", e.translate).unwrap();
                }
            }
        }

        s
    }
}

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36";
static COOKIE: &str = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1";

pub async fn get(word: &str, t: &str) -> Result<Vec<Word>, reqwest::Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://dict.hjenglish.com/jp/{}/{}", t, word))
        .header("User-Agent", USER_AGENT)
        .header("Cookie", COOKIE)
        .send()
        .await?;

    let text = r.text().await?;

    // println!("{}", text);
    Ok(parse(&text))
}

fn parse_detail(element: ElementRef) -> Vec<Detail> {
    let mut eps: Vec<Detail> = vec![];

    let detail_selector = Selector::parse(".word-details-pane-content .word-details-item").unwrap();
    let details = element.select(&detail_selector);
    for detail in details {
        let source_selector = Selector::parse(".detail-source").unwrap();
        let source = match detail.select(&source_selector).next() {
            None => "unknown".to_string(),
            Some(v) => v.text().collect::<Vec<_>>().join(" "),
        };

        let dl_selector = Selector::parse(".word-details-item-content .detail-groups dl").unwrap();
        let dls = detail.select(&dl_selector);

        for dl in dls {
            let attr_selector = Selector::parse("dt").unwrap();
            let attr = match dl.select(&attr_selector).next() {
                None => "unknown".to_string(),
                Some(v) => v.text().collect::<Vec<_>>().join(" ").trim().to_string(),
            };

            let mut d = Detail {
                attribute: attr,
                explains: vec![],
                source: source.clone(),
            };

            let dd_selector = Selector::parse("dd").unwrap();
            let dds = dl.select(&dd_selector);

            for dd in dds {
                let explain_selector = Selector::parse("h3 p").unwrap();
                let mut explain_str = "".to_string();
                for explain in dd.select(&explain_selector) {
                    explain_str += explain.text().collect::<Vec<_>>().join(" ").as_str();
                }

                let mut ep = ExplainsAndExample {
                    explain: explain_str.split_whitespace().collect::<Vec<_>>().join(" "),
                    examples: vec![],
                };
                let example_selector = Selector::parse("ul li").unwrap();
                for example in dd.select(&example_selector) {
                    let from_selector = Selector::parse(".def-sentence-from").unwrap();
                    let to_selector = Selector::parse(".def-sentence-to").unwrap();

                    let from = match example.select(&from_selector).next() {
                        None => "".to_string(),
                        Some(v) => v.text().collect::<Vec<_>>().join(" "),
                    };

                    let to = match example.select(&to_selector).next() {
                        None => "".to_string(),
                        Some(v) => v.text().collect::<Vec<_>>().join(" "),
                    };

                    ep.examples.push(Example {
                        original: from.split_whitespace().collect::<Vec<_>>().join(" "),
                        translate: to.split_whitespace().collect::<Vec<_>>().join(" "),
                    });
                }

                d.explains.push(ep);
            }

            eps.push(d);
        }
    }

    return eps;
}

fn parse_simple(element: ElementRef) -> Vec<Simple> {
    let mut sps: Vec<Simple> = vec![];

    let simple_selector = Selector::parse(".simple").unwrap();

    let simples = element.select(&simple_selector);

    for simple in simples {
        let attributes_selector = Selector::parse("h2").unwrap();
        let mut attributes = simple.select(&attributes_selector);

        let attribute = match attributes.next() {
            None => String::from(""),
            Some(x) => x.inner_html(),
        };

        if attribute == "" {
            let definition_selector = Selector::parse("span.simple-definition").unwrap();

            let definition = match simple.select(&definition_selector).next() {
                None => continue,
                Some(v) => v,
            };

            let html = definition
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            if html != "" {
                sps.push(Simple {
                    attribute: "".to_string(),
                    explains: vec![html],
                });
            }
            continue;
        }

        let list_selector = Selector::parse("ul").unwrap();
        let list = simple.select(&list_selector);

        for li in list {
            let mut sp = Simple {
                attribute: attribute.clone(),
                explains: vec![],
            };

            let li_selector = Selector::parse("li").unwrap();
            let lis = li.select(&li_selector);

            for li in lis {
                let mut li_text = String::new();
                for child in li.children() {
                    if let Some(text_node) = child.value().as_text() {
                        let trimmed = text_node.trim();
                        if !trimmed.is_empty() {
                            li_text.push_str(trimmed);
                        }
                    }
                }

                if li_text.len() == 0 {
                    continue;
                }

                sp.explains.push(li_text);
            }

            sps.push(sp);
        }
    }
    return sps;
}

fn parse(text: &str) -> Vec<Word> {
    let mut ws: Vec<Word> = vec![];

    let q = scraper::Html::parse_document(&text);

    let selector = scraper::Selector::parse(".word-details-pane").unwrap();

    let res = q.select(&selector);

    for element in res {
        let word_selector = Selector::parse(".word-text h2").unwrap();
        let pronounce_selector = Selector::parse(".pronounces").unwrap();
        let pronounce = element.select(&pronounce_selector).next().unwrap();
        let katakana_selector = Selector::parse("span").unwrap();
        let audio_selector = Selector::parse(".word-audio").unwrap();
        let audio = pronounce
            .select(&audio_selector)
            .next()
            .unwrap()
            .value()
            .attr("data-src")
            .unwrap();

        let mut katakana = String::new();

        for p in pronounce.select(&katakana_selector) {
            katakana.push_str(p.text().collect::<Vec<_>>().join("").trim());
        }

        ws.push(Word {
            word: element
                .select(&word_selector)
                .next()
                .unwrap()
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string(),
            katakana: katakana,
            audio_url: audio.to_string(),
            simple: parse_simple(element),
            detail: parse_detail(element),
        });
    }

    return ws;
}

#[cfg(test)]
mod tes {
    use std::fs;

    use crate::jp::parse;

    #[tokio::test]
    async fn run_parse() {
        let test1 = fs::read_to_string("assets/test_data/jpcn.html.txt").unwrap();
        let test2 = fs::read_to_string("assets/test_data/cnjp.html.txt").unwrap();

        println!("{:?}", parse(test1.as_str()));
        println!("{:?}", parse(test2.as_str()));

        for v in parse(test1.as_str()) {
            println!("{}\n", v.markdown())
        }

        for v in parse(test2.as_str()) {
            println!("{}\n", v.markdown())
        }
        // println!("{:?}", get("你好", "cj").await.unwrap());
    }
}
