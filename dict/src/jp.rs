use crate::{
    en::{AttrOrEmpty, COOKIE, StringOrEmpty, TrimText, USER_AGENT},
    error::Error,
};
use scraper::{ElementRef, Selector};
use std::fmt::Write;
use url::form_urlencoded;

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

            for example in &detail.explains {
                writeln!(s, "    - {}  ", example.explain).unwrap();

                for e in &example.examples {
                    writeln!(s, "      - {}  ", e.original).unwrap();
                    writeln!(s, "        {}  ", e.translate).unwrap();
                }
            }
        }

        s
    }
}

pub async fn get(word: &str, t: &str) -> Result<Vec<Word>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!(
            "https://dict.hujiang.com/jp/{}/{}",
            t,
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

fn parse_detail(element: ElementRef) -> Vec<Detail> {
    let mut eps: Vec<Detail> = vec![];

    let details = element.select(sel!(".word-details-pane-content .word-details-item"));
    for detail in details {
        let source = detail.select(sel!(".detail-source")).string_or_empty();

        let dls = detail.select(sel!(".word-details-item-content .detail-groups dl"));

        for dl in dls {
            let attr = dl.select(sel!("dt")).string_or_empty();

            let mut d = Detail {
                attribute: attr,
                explains: vec![],
                source: source.clone(),
            };

            let dds = dl.select(sel!("dd"));

            for dd in dds {
                let mut ep = ExplainsAndExample {
                    explain: dd
                        .select(sel!("h3 p"))
                        .map(|x| x.trim_text())
                        .collect::<Vec<_>>()
                        .join(""),
                    examples: vec![],
                };

                for example in dd.select(sel!("ul li")) {
                    let from = example.select(sel!(".def-sentence-from")).string_or_empty();

                    let to = example.select(sel!(".def-sentence-to")).string_or_empty();

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

    eps
}

fn parse_simple(element: ElementRef) -> Vec<Simple> {
    let mut sps: Vec<Simple> = vec![];

    let simples = element.select(sel!(".simple"));

    for simple in simples {
        let mut attributes = simple.select(sel!("h2"));

        let attribute = attributes.string_or_empty();

        for li in simple.select(sel!("ul")) {
            let mut sp = Simple {
                attribute: attribute.clone(),
                explains: vec![],
            };

            let lis = li.select(sel!("li"));

            for li in lis {
                let li_text = li.trim_text();
                if li_text.is_empty() {
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
    sps
}

fn parse(text: &str) -> Vec<Word> {
    let mut ws: Vec<Word> = vec![];

    let mut q = scraper::Html::parse_document(text);

    // remove useless nodes
    for selector_str in [".simple ul li span"] {
        let selector = Selector::parse(selector_str).unwrap();
        for id in q
            .select(&selector)
            .map(|p_node| p_node.id())
            .collect::<Vec<_>>()
        {
            q.tree.get_mut(id).unwrap().detach();
        }
    }

    let res = q.select(sel!(".word-details-pane"));

    for element in res {
        let mut w = Word {
            word: element.select(sel!(".word-text h2")).string_or_empty(),
            katakana: "".to_string(),
            audio_url: "".to_string(),
            simple: parse_simple(element),
            detail: parse_detail(element),
        };

        if let Some(pronounce) = element.select(sel!(".pronounces")).next() {
            w.audio_url = pronounce
                .select(sel!(".word-audio"))
                .attr_or_empty("data-src");
            w.katakana = pronounce
                .select(sel!("span"))
                .map(|x| x.trim_text())
                .collect::<Vec<_>>()
                .join("");
        }

        ws.push(w);
    }

    ws
}

#[cfg(test)]
mod tes {
    use crate::jp::{get, parse};
    use std::fs;

    #[tokio::test]
    async fn run_parse() {
        let test1 = fs::read_to_string("../assets/test_data/jpcn.html.txt").unwrap();
        let test2 = fs::read_to_string("../assets/test_data/cnjp.html.txt").unwrap();

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
