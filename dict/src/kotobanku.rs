use scraper::Selector;
use url::form_urlencoded;

use crate::error::Error;

pub async fn get(str: &str) -> Result<Vec<String>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!(
            "https://kotobank.jp/word/{}",
            form_urlencoded::byte_serialize(str.as_bytes()).collect::<String>()
        ))
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

    let mut q = scraper::Html::parse_fragment(&text);

    let kiji_selector = Selector::parse("#mainArea article").unwrap();

    let kijis = q.select(&kiji_selector);
    let mut ids: Vec<ego_tree::NodeId> = vec![];

    for kiji in kijis {
        let kyujin_selector = Selector::parse(".kyujinbox-ad").unwrap();
        let source_selector = Selector::parse(".source").unwrap();

        let mut collect_append = |selector: Selector, parent: bool| {
            let mut node_ids = kiji
                .select(&selector)
                .map(|p_node| {
                    if parent {
                        p_node.parent().unwrap().id()
                    } else {
                        p_node.id()
                    }
                })
                .collect::<Vec<_>>();
            ids.append(&mut node_ids);
        };

        collect_append(kyujin_selector, false);
        collect_append(source_selector, false);
    }

    for id in ids {
        q.tree.get_mut(id).unwrap().detach();
    }

    let mut results = Vec::new();

    for kiji in q.select(&kiji_selector) {
        let output = kiji
            .inner_html()
            .replace("<a", "<span")
            .replace("</a", "</span");

        let text = html2text::config::plain_no_decorate()
            .no_link_wrapping()
            .allow_width_overflow()
            .no_table_borders()
            .link_footnotes(false)
            .string_from_read(output.as_bytes(), 9999)
            .unwrap();
        results.push(text);
    }

    Ok(results)
}

#[cfg(test)]
mod test {
    use crate::kotobanku::get;

    #[tokio::test]
    async fn get_test() {
        for v in get("子供").await.unwrap() {
            println!("{}", v);
        }
    }
}
