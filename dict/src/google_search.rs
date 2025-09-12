use crate::error::Error;
use dom_smoothie::Readability;
use log::{info, warn};
use reqwest::Url;

pub static USER_AGENT: &str = "Lynx/3.8.1";

// curl -H "User-Agent: Lynx/3.8.1" "https://www.google.com/search?q=hello"

/*
curl -G "https://www.google.com/search" \
  --data-urlencode "q=生意気　意味" \
  --data-urlencode "filter=0" \
  --data-urlencode "start=0" \
  --data-urlencode "asearch=arc" \
  --data-urlencode "async=arc_id:srp_Ez6mgiQ7CjInnjwrnLE06PI_100,use_ac:true,_fmt:prog" \
  --data-urlencode "ie=UTF-8" \
  --data-urlencode "oe=UTF-8" \
  --data-urlencode "hl=ja-JP" \
  --data-urlencode "lr=lang_ja" \
  --data-urlencode "cr=countryJA" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
 */

pub async fn getv2(word: &str) -> Result<Vec<Body>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://www.google.com/search?q={}", word))
        .query(&[
            ("q", word),
            ("filter", "0"),
            ("start", "0"),
            ("asearch", "arc"),
            (
                "async",
                "arc_id:srp_Ez6mgiQ7CjInnjwrnLE06PI_100,use_ac:true,_fmt:prog",
            ),
            ("ie", "UTF-8"),
            ("oe", "UTF-8"),
            ("hl", "ja-JP"),
            ("lr", "lang_ja"),
            ("cr", "countryJA"),
        ])
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36")
        .send()
        .await?;

    let status = r.status();
    let body = trim_to_html(r.text().await?);

    if !status.is_success() {
        return Err(Error {
            status: Some(status),
            message: body,
        })?;
    }

    let q = scraper::Html::parse_document(&body);

    let a_selector = scraper::Selector::parse("a").unwrap();
    let h3_selector = scraper::Selector::parse("h3").unwrap();
    let div_selector = scraper::Selector::parse("div").unwrap();

    let mut ws: Vec<Body> = vec![];

    for element in q.select(&div_selector) {
        match element.attr("data-sncf") {
            Some(path) if path == "1" => {}
            _ => continue,
        };

        ws.push(Body::Content(element.text().collect::<Vec<_>>().join("")));
    }

    for element in q.select(&a_selector) {
        let path = match element.attr("href") {
            Some(path) => path,
            None => continue,
        };

        let url = match Url::parse(path) {
            Ok(url) => url,
            Err(_) => continue,
        };

        let title = match element.select(&h3_selector).next() {
            Some(title) => title.text().collect::<Vec<_>>().join(""),
            None => "".to_string(),
        };

        ws.push(Body::Link(SearchLink {
            url: url.to_string(),
            title,
        }));
    }

    Ok(ws)
}

pub async fn get(word: &str) -> Result<Vec<Body>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://www.google.com/search?q={}", word))
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    let status = r.status();

    let text = r.text().await?;

    println!("{}", text);
    info!("google search raw result: {}, status: {}", text, status);

    if !status.is_success() {
        return Err(Error {
            status: Some(status),
            message: text,
        })?;
    }

    Ok(parse(&text))
}

#[derive(Debug)]
pub enum Body {
    Content(String),
    Link(SearchLink),
}

#[derive(Debug)]
pub struct SearchLink {
    pub url: String,
    pub title: String,
}

pub(crate) fn clean_text_lines(input: &str) -> String {
    input
        .lines()
        .filter_map(|line| {
            let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
            if line.is_empty() { None } else { Some(line) }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug)]
pub struct SearchError(String);

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<reqwest::Error> for SearchError {
    fn from(value: reqwest::Error) -> Self {
        SearchError(value.to_string())
    }
}

impl From<dom_smoothie::ReadabilityError> for SearchError {
    fn from(value: dom_smoothie::ReadabilityError) -> Self {
        SearchError(value.to_string())
    }
}

impl SearchLink {
    pub async fn get_raw_page(&self) -> Result<(String, String), SearchError> {
        let r = reqwest::Client::builder()
            .build()?
            .get(&self.url)
            .header("User-Agent", USER_AGENT)
            // .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .send()
            .await?;

        let bytes = r.text().await?;

        let cfg = dom_smoothie::Config {
            text_mode: dom_smoothie::TextMode::Formatted,
            disable_json_ld: true,
            candidate_select_mode: dom_smoothie::CandidateSelectMode::DomSmoothie,
            ..Default::default()
        };

        let mut dr = Readability::new(bytes.clone(), Some(&self.url), Some(cfg))?;

        match dr.parse() {
            Ok(drc) => Ok((drc.title, drc.text_content.to_string())),
            Err(e) => {
                warn!("readability parse error: {}, fallback to html2text", e);

                Ok((
                    self.title.clone(),
                    html2text::config::plain()
                        .no_link_wrapping()
                        .allow_width_overflow()
                        .no_table_borders()
                        .link_footnotes(false)
                        .string_from_read(
                            &bytes
                                .clone()
                                .replace("<a", "<span")
                                .replace("</a>", "</span>")
                                .as_bytes()[..],
                            9999,
                        )
                        .unwrap(),
                ))
            }
        }
    }
}

fn parse(text: &str) -> Vec<Body> {
    let mut ws: Vec<Body> = vec![];

    let q = scraper::Html::parse_document(&text);

    let selector = scraper::Selector::parse("a").unwrap();

    let res = q.select(&selector);

    for element in res {
        let path = match element.attr("href") {
            Some(path) => path,
            None => continue,
        };

        let url = match Url::parse(&format!("https://www.google.com{}", path)) {
            Ok(url) => url,
            Err(_) => continue,
        };
        let query = match url.query_pairs().find(|x| x.0 == "q") {
            Some(query) => query,
            None => continue,
        };

        let ret = match Url::parse(&query.1) {
            Ok(url) => {
                if url.host_str().unwrap_or("").ends_with("google.com") {
                    continue;
                }
                url
            }
            Err(_) => continue,
        };

        let text = element.text().collect::<Vec<_>>().join("");
        ws.push(Body::Link(SearchLink {
            url: ret.to_string(),
            title: clean_text_lines(&text.replace("\n", " ")),
        }));
    }

    ws
}

fn trim_to_html(raw: String) -> String {
    let start = raw.find('<').unwrap_or(0);
    let end = raw.rfind('>').unwrap_or(raw.len() - 1);
    if start < end {
        raw[start..=end].to_string()
    } else {
        raw
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use dom_smoothie::Readability;

    use crate::google_search::{Body, clean_text_lines, getv2, parse, trim_to_html};

    #[test]
    fn test() {
        let text = fs::read_to_string("../assets/test_data/google_search.html.txt").unwrap();
        println!("{:?}", parse(&text));
    }

    #[test]
    fn test_eow() {
        let text = fs::read_to_string("../assets/test_data/eow.html.txt")
            .unwrap()
            .replace("<a", "<a")
            .replace("</a", "</a");

        let cfg = dom_smoothie::Config {
            text_mode: dom_smoothie::TextMode::Formatted,
            disable_json_ld: true,
            candidate_select_mode: dom_smoothie::CandidateSelectMode::Readability,
            ..Default::default()
        };

        let mut dr = Readability::new(
            text.clone(),
            Some("https://eow.alc.co.jp/search?q=straw+bale"),
            Some(cfg),
        )
        .unwrap();

        let drc = dr.parse().unwrap();

        println!(
            "text_content: {}",
            clean_text_lines(&drc.text_content.to_string())
        );
        println!("title: {}", drc.title);
        println!("excerpt: {:?}", drc.excerpt);
        println!("site: {:?}", drc.site_name);
        println!("image: {:?}", drc.image);
        println!("url: {:?}", drc.url);
    }

    #[tokio::test]
    async fn test_asyncv2() {
        let v2text = getv2("子供 意味").await.unwrap();
        println!("{:?}", v2text);

        for v in v2text {
            match v {
                Body::Link(link) => {
                    if !link.url.contains("weblio") {
                        continue;
                    }

                    println!("{:?}", link.get_raw_page().await.unwrap());
                    break;
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_trim_html() {
        let text = fs::read_to_string("../assets/test_data/google_search_v2.html.txt").unwrap();
        println!("{:?}", trim_to_html(text));
    }
}
