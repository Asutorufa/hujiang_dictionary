use crate::en::{AttrOrEmpty, COOKIE, StringOrEmpty, TrimText, USER_AGENT};
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
        let source = detail.select(&source_selector).string_or_empty();

        let dl_selector = Selector::parse(".word-details-item-content .detail-groups dl").unwrap();
        let dls = detail.select(&dl_selector);

        for dl in dls {
            let attr_selector = Selector::parse("dt").unwrap();
            let attr = dl.select(&attr_selector).string_or_empty();

            let mut d = Detail {
                attribute: attr,
                explains: vec![],
                source: source.clone(),
            };

            let dd_selector = Selector::parse("dd").unwrap();
            let dds = dl.select(&dd_selector);

            for dd in dds {
                let explain_selector = Selector::parse("h3 p").unwrap();
                let mut ep = ExplainsAndExample {
                    explain: dd
                        .select(&explain_selector)
                        .map(|x| x.trim_text())
                        .collect::<Vec<_>>()
                        .join(""),
                    examples: vec![],
                };

                let example_selector = Selector::parse("ul li").unwrap();
                for example in dd.select(&example_selector) {
                    let from_selector = Selector::parse(".def-sentence-from").unwrap();
                    let to_selector = Selector::parse(".def-sentence-to").unwrap();

                    let from = example.select(&from_selector).string_or_empty();

                    let to = example.select(&to_selector).string_or_empty();

                    ep.examples.push(Example {
                        original: from,
                        translate: to,
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

        let attribute = attributes.string_or_empty();

        let list_selector = Selector::parse("ul").unwrap();

        for li in simple.select(&list_selector) {
            let mut sp = Simple {
                attribute: attribute.clone(),
                explains: vec![],
            };

            let li_selector = Selector::parse("li").unwrap();
            let lis = li.select(&li_selector);

            for li in lis {
                let li_text = li.trim_text();
                if li_text.len() == 0 {
                    continue;
                }

                sp.explains.push(li_text);
            }

            if sp.explains.is_empty() {
                continue;
            }

            sps.push(sp);
        }

        if sps.is_empty() {
            let html = simple.trim_text();
            if !html.is_empty() {
                sps.push(Simple {
                    attribute: attribute.clone(),
                    explains: vec![html],
                });
            }
        }
    }
    return sps;
}

fn parse(text: &str) -> Vec<Word> {
    let mut ws: Vec<Word> = vec![];

    let mut q = scraper::Html::parse_document(&text);

    // remove useless nodes
    for selector_str in vec![".simple ul li span"] {
        let selector = Selector::parse(selector_str).unwrap();
        for id in q
            .select(&selector)
            .map(|p_node| p_node.id())
            .collect::<Vec<_>>()
        {
            q.tree.get_mut(id).unwrap().detach();
        }
    }

    let selector = scraper::Selector::parse(".word-details-pane").unwrap();

    let res = q.select(&selector);

    for element in res {
        let word_selector = Selector::parse(".word-text h2").unwrap();
        let pronounce_selector = Selector::parse(".pronounces").unwrap();
        let katakana_selector = Selector::parse("span").unwrap();
        let audio_selector = Selector::parse(".word-audio").unwrap();

        let mut w = Word {
            word: element.select(&word_selector).string_or_empty(),
            katakana: "".to_string(),
            audio_url: "".to_string(),
            simple: parse_simple(element),
            detail: parse_detail(element),
        };

        if let Some(pronounce) = element.select(&pronounce_selector).next() {
            w.audio_url = pronounce.select(&audio_selector).attr_or_empty("data-src");
            w.katakana = pronounce
                .select(&katakana_selector)
                .map(|x| x.trim_text())
                .collect::<Vec<_>>()
                .join("");
        }

        ws.push(w);
    }

    return ws;
}

#[cfg(test)]
mod tes {
    use std::fs;

    use crate::jp::{get, parse};

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

        println!(
            "{}",
            get("オセロ", "jc")
                .await
                .unwrap()
                .iter()
                .map(|v| v.markdown())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
