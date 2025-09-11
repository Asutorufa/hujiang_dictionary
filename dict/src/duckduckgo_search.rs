use log::info;
use reqwest::Url;

use crate::{
    error::Error,
    google_search::{Body, SearchLink, clean_text_lines},
};

pub static USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";

pub async fn get(word: &str) -> Result<Vec<Body>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://html.duckduckgo.com/html/?q={}", word))
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    /*
        another: https://api.duckduckgo.com/?q=chatgpt&format=json
    */

    let status = r.status();

    let text = r.text().await?;

    info!("duckduckgo search raw result: {}, status: {}", text, status);

    if !status.is_success() {
        return Err(Error {
            status: Some(status),
            message: text,
        })?;
    }

    Ok(parse(&text))
}

fn parse(text: &str) -> Vec<Body> {
    let mut ws: Vec<Body> = vec![];

    let q = scraper::Html::parse_document(&text);

    let selector = scraper::Selector::parse(".result__title .result__a").unwrap();

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
        let query = match url.query_pairs().find(|x| x.0 == "uddg") {
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

    use crate::{
        duckduckgo_search::{get, parse},
        google_search::Body,
    };

    #[test]
    fn test() {
        let text = fs::read_to_string("../assets/test_data/duckduckgo.html.txt").unwrap();
        for v in parse(&text) {
            match v {
                Body::Content(s) => println!("{}", s),
                Body::Link(link) => println!("url: {}, title: {}", link.url, link.title),
            }
        }
    }

    #[tokio::test]
    async fn test_async() {
        let text = get("子供 意味").await.unwrap();

        match &text[0] {
            Body::Content(s) => println!("{}", s),
            Body::Link(link) => println!("{:?}", link.get_raw_page().await.unwrap()),
        }

        for v in text {
            match v {
                Body::Content(s) => println!("{}", s),
                Body::Link(link) => println!("url: {}, title: {}", link.url, link.title),
            }
        }
    }
}
