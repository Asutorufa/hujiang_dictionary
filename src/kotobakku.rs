use scraper::Selector;

pub async fn get(str: &str) -> Result<Vec<String>, reqwest::Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://kotobank.jp/word/{}", str))
        .send()
        .await?;

    let text = r.text().await?;

    let q = scraper::Html::parse_fragment(&text);

    let kiji_selector = Selector::parse("#mainArea article").unwrap();

    let mut results = Vec::new();

    for kiji in q.select(&kiji_selector) {
        let text = html2text::config::plain_no_decorate()
            .no_link_wrapping()
            .allow_width_overflow()
            .no_table_borders()
            .string_from_read(&kiji.inner_html().as_bytes()[..], 9999)
            .unwrap();
        results.push(text);
    }

    Ok(results)
}

#[cfg(test)]
mod test {
    use crate::kotobakku::get;

    #[tokio::test]
    async fn get_test() {
        println!("{:?}", get("子供").await.unwrap());
    }
}
