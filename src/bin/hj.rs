use hjdict::{en, google, jp, kotobanku, kr, weblio};
use std::env::args;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let args = &args().collect::<Vec<_>>()[1..];

    if args.is_empty() || args[0] == "help" || args.len() < 2 {
        println!(
            r#"Usage:
  jc <word> - Japanese to Chinese
  cj <word> - Chinese to Japanese
  en <word> - English to Japanese
  weblio <word> - weblio
  ktbk <word> - コトバンク
  google <target> <words> - Google Translate, eg: google en こんにちは
  googlev1 <target> <words> - Old Google Translate API, eg: googlev1 en こんにちは
  help - show this message"#
        );
        return;
    }

    let word = &args[1..].join(" ");
    let result = match args[0].as_str() {
        "jc" => jp::get(word.as_str(), "jc")
            .await
            .unwrap()
            .iter()
            .map(|x| x.markdown())
            .collect::<Vec<_>>()
            .join("\n"),
        "cj" => jp::get(word.as_str(), "cj")
            .await
            .unwrap()
            .iter()
            .map(|x| x.markdown())
            .collect::<Vec<_>>()
            .join("\n"),
        "kr" => kr::get(word.as_str())
            .await
            .unwrap()
            .iter()
            .map(|x| x.markdown())
            .collect::<Vec<_>>()
            .join("\n"),
        "en" => en::get(word.as_str())
            .await
            .unwrap()
            .iter()
            .map(|x| x.markdown())
            .collect::<Vec<_>>()
            .join("\n"),
        "weblio" => weblio::get(word).await.unwrap().join("\n"),
        "ktbk" => kotobanku::get(word).await.unwrap().join("\n"),
        "google" | "googlev1" => {
            let target = &args[1];
            let words = args[2..].join(" ");

            if words.is_empty() {
                return;
            }

            let query = async |text: &str, src: Option<String>, target: &str| {
                if args[0] == "googlev1" {
                    return google::translate(text, src, target).await;
                } else {
                    return google::translatev2(text, src, target).await;
                }
            };

            query(words.as_str(), None, target)
                .await
                .unwrap()
                .iter()
                .map(|x| x.translation.as_ref())
                .collect::<Vec<_>>()
                .join("")
        }
        _ => {
            format!("Unknown command: {}", args[0])
        }
    };

    println!("{}", result);
}
