use crate::error::Error;
use scraper::Selector;
use std::fmt::Write;

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
pub struct EnglishExplain {
    pub attribute: String,
    pub explains: Vec<String>,
}

#[derive(Debug)]
pub struct Detail {
    pub attribute: String,
    pub explains: Vec<ExplainsAndExample>,
}

#[derive(Debug)]
pub struct Explain {
    pub attribute: String,
    pub explains: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Pronounce {
    pub pronounce: String,
    pub audio_en_url: String,
    pub audio_us_url: String,
}

impl Pronounce {
    pub fn en_pronounce(&self) -> String {
        if let Some(first) = self.audio_en_url.split_whitespace().next() {
            first.to_string()
        } else {
            self.audio_en_url.clone()
        }
    }

    pub fn en_url(&self) -> String {
        if let Some(pos) = self.audio_en_url.find("http") {
            self.audio_en_url[pos..].to_string()
        } else {
            "".to_string()
        }
    }

    pub fn us_pronounce(&self) -> String {
        if let Some(first) = self.audio_us_url.split_whitespace().next() {
            first.to_string()
        } else {
            self.audio_us_url.clone()
        }
    }

    pub fn us_url(&self) -> String {
        if let Some(pos) = self.audio_us_url.find("http") {
            self.audio_us_url[pos..].to_string()
        } else {
            "".to_string()
        }
    }
}

#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub pronounce: Pronounce,
    pub detail: Vec<Detail>,
    pub phrase: Vec<String>,
    pub synonym: Vec<String>,
    pub antonym: Vec<String>,
    pub inflections: Vec<String>,
    pub simple: Vec<String>,
    pub english_explain: Vec<EnglishExplain>,
}

impl Word {
    pub fn markdown(&self) -> String {
        let word = self;

        let mut s = String::new();

        write!(s, "{}\n", word.word).unwrap();
        write!(s, "{}   \n", word.pronounce.pronounce).unwrap();
        write!(
            s,
            r#"
{}
<audio controls controlsList="nodownload" preload="none" src="{}"></audio>
{}
<audio controls controlsList="nodownload" preload="none" src="{}"></audio>
"#,
            word.pronounce.us_pronounce(),
            word.pronounce.us_url(),
            word.pronounce.en_pronounce(),
            word.pronounce.en_url()
        )
        .unwrap();

        if !word.simple.is_empty() {
            s.push_str("\n- simple explain\n");
            for explain in &word.simple {
                write!(s, "  - {}\n", explain).unwrap();
            }
        }

        if !word.detail.is_empty() {
            s.push_str("\n- More Detail\n");
            for detail in &word.detail {
                write!(s, "  - {}  \n", detail.attribute).unwrap();
                for explain_ex in &detail.explains {
                    write!(s, "    - {}  \n", explain_ex.explain).unwrap();
                    for example in &explain_ex.examples {
                        write!(s, "      - {}  \n", example.original).unwrap();
                        write!(s, "        {}  \n", example.translate).unwrap();
                    }
                }
            }
        }

        if !word.english_explain.is_empty() {
            s.push_str("\n- English Explain\n");
            for eng in &word.english_explain {
                write!(s, "  - {}  \n", eng.attribute).unwrap();
                for explain in &eng.explains {
                    write!(s, "    - {}  \n", explain).unwrap();
                }
            }
        }

        if !word.inflections.is_empty() {
            s.push_str("\n- Inflections\n");
            for infl in &word.inflections {
                write!(s, "  - {}  \n", infl).unwrap();
            }
        }

        if !word.phrase.is_empty() {
            s.push_str("\n- Phrase\n");
            for p in &word.phrase {
                write!(s, "  - {}  \n", p).unwrap();
            }
        }

        if !word.synonym.is_empty() {
            s.push_str("\n- Synonym\n");
            for syn in &word.synonym {
                write!(s, "  - {}  \n", syn).unwrap();
            }
        }

        if !word.antonym.is_empty() {
            s.push_str("\n- Antonym\n");
            for ant in &word.antonym {
                write!(s, "  - {}  \n", ant).unwrap();
            }
        }

        s
    }
}

pub(crate) trait StringOrEmpty {
    fn string_or_empty(&mut self) -> String;
}

pub(crate) trait TrimText {
    fn trim_text(&self) -> String;
}

pub(crate) trait AttrOrEmpty {
    fn attr_or_empty(&mut self, attr: &str) -> String;
}

impl TrimText for scraper::element_ref::ElementRef<'_> {
    fn trim_text(&self) -> String {
        self.text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl StringOrEmpty for scraper::element_ref::Select<'_, '_> {
    fn string_or_empty(&mut self) -> String {
        match self.next() {
            Some(x) => x
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            None => "".to_string(),
        }
    }
}

impl AttrOrEmpty for scraper::element_ref::Select<'_, '_> {
    fn attr_or_empty(&mut self, attr: &str) -> String {
        match self.next() {
            Some(x) => x.attr(attr).unwrap_or(&"").to_string(),
            None => "".to_string(),
        }
    }
}

pub static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.81 Safari/537.36";
pub static COOKIE: &str = "HJ_UID=0f406091-be97-6b64-f1fc-f7b2470883e9; HJ_CST=1; HJ_CSST_3=1;TRACKSITEMAP=3%2C; HJ_SID=393c85c7-abac-f408-6a32-a1f125d7e8c6; _REF=; HJ_SSID_3=4a460f19-c0ae-12a7-8e86-6e360f69ec9b; _SREF_3=; HJ_CMATCH=1";

pub async fn get(word: &str) -> Result<Vec<Word>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://dict.hjenglish.com/w/{}", word))
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

pub fn parse(text: &str) -> Vec<Word> {
    let mut ws: Vec<Word> = vec![];

    let q = scraper::Html::parse_document(text);

    let word_details_pane_selector = scraper::Selector::parse(".word-details-pane").unwrap();

    for w in q.select(&word_details_pane_selector) {
        let mut word = Word {
            word: "".to_string(),
            pronounce: Pronounce {
                pronounce: "".to_string(),
                audio_en_url: "".to_string(),
                audio_us_url: "".to_string(),
            },
            simple: vec![],
            phrase: vec![],
            inflections: vec![],
            synonym: vec![],
            antonym: vec![],
            detail: vec![],
            english_explain: vec![],
        };

        word.word = w
            .select(&scraper::Selector::parse(".word-text h2").unwrap())
            .string_or_empty();

        if let Some(pronounces) = w
            .select(&scraper::Selector::parse(".pronounces").unwrap())
            .next()
        {
            if let Some(en) = pronounces
                .select(&scraper::Selector::parse(".pronounce-value-en").unwrap())
                .next()
            {
                word.pronounce.audio_en_url = format!(
                    "{} {}",
                    en.text().collect::<Vec<_>>().join("").trim(),
                    pronounces
                        .select(&scraper::Selector::parse(".word-audio-en").unwrap())
                        .attr_or_empty("data-src")
                );
                word.pronounce.audio_us_url = format!(
                    "{} {}",
                    pronounces
                        .select(&scraper::Selector::parse(".pronounce-value-us").unwrap())
                        .string_or_empty(),
                    pronounces
                        .select(&scraper::Selector::parse(".word-audio").unwrap())
                        .attr_or_empty("data-src")
                );
            } else {
                word.pronounce.pronounce = pronounces
                    .select(&Selector::parse("span").unwrap())
                    .string_or_empty();
                word.pronounce.audio_us_url = pronounces
                    .select(&Selector::parse(".word-audio").unwrap())
                    .attr_or_empty("data-src")
            }
        }

        for s in w.select(&Selector::parse(".simple p").unwrap()) {
            word.simple.push(s.trim_text());

            for s in s.select(&Selector::parse(".simple-definition a").unwrap()) {
                word.simple.push(s.trim_text());
            }
        }

        if let Some(word_detail_item) = w
            .select(&Selector::parse(".word-details-item-content").unwrap())
            .next()
        {
            for s in word_detail_item.select(&Selector::parse(".phrase-items li").unwrap()) {
                word.phrase.push(s.trim_text());
            }

            for s in word_detail_item.select(&Selector::parse(".inflections-items li").unwrap()) {
                word.inflections.push(s.trim_text());
            }

            for s in word_detail_item.select(&Selector::parse(".syn table tbody tr td a").unwrap())
            {
                word.synonym.push(s.trim_text());
            }

            for s in word_detail_item.select(&Selector::parse(".ant table tbody tr td a").unwrap())
            {
                word.antonym.push(s.trim_text());
            }

            for ed in word_detail_item.select(&Selector::parse(".enen-groups dl").unwrap()) {
                let mut explain = EnglishExplain {
                    attribute: ed.select(&Selector::parse("dt").unwrap()).string_or_empty(),
                    explains: vec![],
                };

                for e in ed.select(&Selector::parse("dd").unwrap()) {
                    explain.explains.push(e.trim_text());
                }

                word.english_explain.push(explain);
            }

            for dl in word_detail_item.select(&Selector::parse(".detail-groups dl").unwrap()) {
                let mut detail = Detail {
                    attribute: dl.select(&Selector::parse("dt").unwrap()).string_or_empty(),
                    explains: vec![],
                };

                for dd in dl.select(&Selector::parse("dd").unwrap()) {
                    let mut explain = ExplainsAndExample {
                        explain: dd.select(&Selector::parse("h3").unwrap()).string_or_empty(),
                        examples: vec![],
                    };

                    for li in dd.select(&Selector::parse("ul li").unwrap()) {
                        explain.examples.push(Example {
                            original: li
                                .select(&Selector::parse(".def-sentence-from").unwrap())
                                .string_or_empty(),
                            translate: li
                                .select(&Selector::parse(".def-sentence-to").unwrap())
                                .string_or_empty(),
                        });
                    }

                    detail.explains.push(explain);
                }

                word.detail.push(detail);
            }
        }

        ws.push(word);
    }

    ws
}

#[cfg(test)]
mod tes {
    use std::fs;

    use crate::en::{get, parse};

    #[tokio::test]
    async fn run_parse() {
        let test1 = fs::read_to_string("../assets/test_data/en.html.txt").unwrap();
        println!("{:?}", parse(test1.as_str()));

        for v in parse(test1.as_str()) {
            println!("{}", v.markdown());
        }

        println!("{:?}", get("message").await.unwrap());
    }
}
