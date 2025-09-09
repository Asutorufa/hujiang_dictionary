use crate::error::Error;
use reqwest::Url;

pub static USER_AGENT: &str = "Lynx/3.8.1";

// curl -H "User-Agent: Lynx/3.8.1" "https://www.google.com/search?q=hello"

pub async fn get(word: &str) -> Result<Vec<SearchLink>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://www.google.com/search?q={}", word))
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    let status = r.status();

    let text = r.text().await?;

    if status != 200 {
        return Err(Error {
            status: Some(status),
            message: text,
        })?;
    }

    Ok(parse(&text))
}

#[derive(Debug)]
pub struct SearchLink {
    pub url: String,
    pub title: String,
}

fn clean_text_lines(input: &str) -> String {
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

fn parse(text: &str) -> Vec<SearchLink> {
    let mut ws: Vec<SearchLink> = vec![];

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
        ws.push(SearchLink {
            url: ret.to_string(),
            title: clean_text_lines(&text.replace("\n", " ")),
        });
    }

    ws
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::google_search::{get, parse};

    #[test]
    fn test() {
        let text = fs::read_to_string("../assets/test_data/google_search.html.txt").unwrap();
        println!("{:?}", parse(&text));
    }

    #[tokio::test]
    async fn test_async() {
        let text = get("子供 意味").await.unwrap();

        println!("{:?}", text[0].get_raw_page().await.unwrap());
        println!("{:?}", text[1].get_raw_page().await.unwrap());

        for v in text {
            println!("url: {}, title: {}", v.url, v.title);
        }
    }
}
