use scraper::{Html, Selector};

pub async fn get(str: &str) -> Result<Vec<String>, reqwest::Error> {
    let r = reqwest::Client::builder()
        .build()?
        .get(format!("https://www.weblio.jp/content/{}", str))
        .send()
        .await?;

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
        let footnote_selector = Selector::parse(".footNote").unwrap();
        let footnote_b_selector = Selector::parse(".footNoteB").unwrap();
        let reflist_selector = Selector::parse(".reflist").unwrap();
        let sister_projects_selector = Selector::parse(".sister-projects").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let noscript_selector = Selector::parse("noscript").unwrap();
        let citation_selector = Selector::parse(".citation").unwrap();
        let sidebox_selector = Selector::parse(".side-box").unwrap();
        let external_selector = Selector::parse(".external").unwrap();
        let navbox_selector = Selector::parse(".navbox").unwrap();
        let reference_selector = Selector::parse(".reference").unwrap();
        let cite_bracket_selector = Selector::parse(".cite-bracket").unwrap();

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

        collect_append(footnote_selector, false);
        collect_append(footnote_b_selector, false);
        collect_append(reflist_selector, false);
        collect_append(sister_projects_selector, false);
        collect_append(citation_selector, true);
        collect_append(img_selector, false);
        collect_append(noscript_selector, false);
        collect_append(sidebox_selector, false);
        collect_append(external_selector, true);
        collect_append(navbox_selector, false);
        collect_append(reference_selector, false);
        collect_append(cite_bracket_selector, false);
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
            .string_from_read(&output.as_bytes()[..], 9999)
            .unwrap();

        results.push(text);
    }

    results
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, OpenOptions},
        io::{self, Write},
    };

    use crate::weblio::{get, parse};

    fn append_to_file(filename: &str, content: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }

    #[tokio::test]
    async fn get_test() {
        let test = fs::read_to_string("assets/test_data/weblio.html.txt").unwrap();

        for v in parse(&test) {
            println!("{}", v);
        }
        println!("{:?}", get("子供").await.unwrap());
        println!("{:?}", get("嘆く").await.unwrap());
    }
}
