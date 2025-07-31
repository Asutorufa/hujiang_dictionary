use scraper::Selector;

pub async fn get(str: &str) -> Result<(), reqwest::Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://www.weblio.jp/content/{}", str))
        .send()
        .await?;

    let text = r.text().await?;

    let q = scraper::Html::parse_fragment(&text);

    let kiji_selector = Selector::parse(".kiji").unwrap();

    for kiji in q.select(&kiji_selector) {
        let text = html2text::config::plain_no_decorate()
            .no_link_wrapping()
            .allow_width_overflow()
            .no_table_borders()
            .string_from_read(&kiji.inner_html().as_bytes()[..], 9999)
            .unwrap();
        println!("----------- {}", text);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::weblio::get;

    #[tokio::test]
    async fn get_test() {
        get("子供").await.unwrap();
    }
}
