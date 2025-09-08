use reqwest::Url;

pub static USER_AGENT: &str = "Lynx/3.8.1";

// curl -H "User-Agent: Lynx/3.8.1" "https://www.google.com/search?q=hello"

pub async fn get(word: &str) -> Result<Vec<SearchLink>, reqwest::Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://www.google.com/search?q={}", word))
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    let text = r.text().await?;

    Ok(parse(&text))
}

#[derive(Debug)]
pub struct SearchLink {
    pub url: String,
    pub title: String,
}

impl SearchLink {
    pub async fn get_raw_page(&self) -> Result<String, reqwest::Error> {
        let r = reqwest::Client::builder()
            .build()?
            .get(&self.url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .send()
            .await?;

        let text = r.text().await?;

        Ok(text)
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
            Ok(url) => url,
            Err(_) => continue,
        };

        let text = element.text().collect::<Vec<_>>().join("");
        ws.push(SearchLink {
            url: ret.to_string(),
            title: text,
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
        parse(&text);
    }

    #[tokio::test]
    async fn test_async() {
        let text = get("子供の意味").await.unwrap();
        println!("{:?}", text);
        println!("{:?}", text[0].get_raw_page().await.unwrap());
    }
}
