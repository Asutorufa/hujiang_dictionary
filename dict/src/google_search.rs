use crate::error::Error;
use log::info;
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
    let body = r.text().await?;

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

impl SearchLink {
    pub async fn get_raw_page(&self) -> Result<String, reqwest::Error> {
        let r = reqwest::Client::builder()
            .build()?
            .get(&self.url)
            .header("User-Agent", USER_AGENT)
            // .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .send()
            .await?;

        let bytes = r
            .text()
            .await?
            .replace("<a", "<span")
            .replace("</a>", "</span>");

        let text = html2text::config::plain()
            .no_link_wrapping()
            .allow_width_overflow()
            .no_table_borders()
            .link_footnotes(false)
            .no_table_borders()
            .string_from_read(&bytes.as_bytes()[..], 9999)
            .unwrap();

        Ok(clean_text_lines(&text))
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

#[cfg(test)]
mod test {
    use std::fs;

    use crate::google_search::{getv2, parse};

    #[test]
    fn test() {
        let text = fs::read_to_string("../assets/test_data/google_search.html.txt").unwrap();
        println!("{:?}", parse(&text));
    }

    #[tokio::test]
    async fn test_asyncv2() {
        let v2text = getv2("子供 意味").await.unwrap();
        println!("{:?}", v2text);

        // let text = get("子供 意味").await.unwrap();

        // println!("{:?}", text[0].get_raw_page().await.unwrap());
        // println!("{:?}", text[1].get_raw_page().await.unwrap());

        // for v in text {
        //     println!("url: {}, title: {}", v.url, v.title);
        // }
    }
}
