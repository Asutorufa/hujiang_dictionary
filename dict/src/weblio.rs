use scraper::{Html, Selector};
use url::form_urlencoded;

use crate::error::Error;

pub async fn get(str: &str) -> Result<Vec<String>, Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!(
            "https://www.weblio.jp/content/{}",
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

    Ok(parse(&text))
}

pub fn parse(text: &str) -> Vec<String> {
    let mut q = Html::parse_document(text);

    let kiji_selector = Selector::parse(".kiji").unwrap();

    let mut results = Vec::new();

    let kijis = q.select(&kiji_selector);
    let mut ids: Vec<ego_tree::NodeId> = vec![];

    for kiji in kijis {
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

        for (selector, parent) in vec![
            (".footNote", false),
            (".footNoteB", false),
            (".reflist", false),
            (".sister-projects", false),
            (".citation", true),
            ("img", false),
            ("noscript", false),
            (".side-box", false),
            (".external", false),
            (".navbox", false),
            (".reference", false),
            (".cite-bracket", false),
        ] {
            let s = Selector::parse(selector).unwrap();
            collect_append(s, parent);
        }
    }

    for id in ids {
        q.tree.get_mut(id).unwrap().detach();
    }

    let kijis = q.select(&kiji_selector);

    for kiji in kijis {
        let output = kiji
            .inner_html()
            .replace("<a", "<span")
            .replace("</a", "</span");

        let text = html2text::config::plain_no_decorate()
            .no_link_wrapping()
            .allow_width_overflow()
            .no_table_borders()
            .link_footnotes(false)
            .no_table_borders()
            .string_from_read(output.as_bytes(), 9999)
            .unwrap();

        results.push(text);
    }

    results
}

#[cfg(test)]
mod test {
    use std::fs::{self};

    use crate::weblio::{get, parse};

    // fn append_to_file(filename: &str, content: &str) -> io::Result<()> {
    //     let mut file = OpenOptions::new()
    //         .append(true)
    //         .create(true)
    //         .open(filename)?;

    //     file.write_all(content.as_bytes())?;

    //     Ok(())
    // }

    #[tokio::test]
    async fn get_test() {
        let test = fs::read_to_string("../assets/test_data/weblio.html.txt").unwrap();

        for v in parse(&test) {
            println!("{}", v);
        }
        println!("{:?}", get("子供").await.unwrap());
        println!("{:?}", get("嘆く").await.unwrap());
    }
}
