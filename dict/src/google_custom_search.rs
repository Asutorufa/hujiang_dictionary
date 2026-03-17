use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GoogleSearchResponse {
    pub items: Option<Vec<GoogleSearchItem>>,
}

#[derive(Deserialize, Debug)]
pub struct GoogleSearchItem {
    pub title: String,
    pub link: String,
    pub snippet: Option<String>,
}

pub async fn search(
    query: &str,
    api_key: &str,
    cx: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://customsearch.googleapis.com/customsearch/v1";

    let client = Client::new();
    let res = client
        .get(url)
        .query(&[("key", api_key), ("cx", cx), ("q", query)])
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Google API Error: {} - {}", status, body).into());
    }

    let response: GoogleSearchResponse = res.json().await?;
    let mut results = Vec::new();

    if let Some(items) = response.items {
        for item in items.into_iter().take(5) {
            if let Some(snippet) = item.snippet {
                results.push(format!("Title: {}\nLink: {}\nSnippet: {}\n", item.title, item.link, snippet));
            }
        }
    }

    if results.is_empty() {
        Ok("No results found.".to_string())
    } else {
        Ok(results.join("\n"))
    }
}
