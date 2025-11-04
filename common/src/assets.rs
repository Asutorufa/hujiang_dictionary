use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "config/"]
pub struct Assets;

#[cfg(test)]
mod test {
    use crate::assets::Assets;

    #[test]
    fn test() {
        for v in Assets::iter() {
            let f = Assets::get(&v).unwrap();
            println!(
                "{}: {}, {}",
                v,
                v.ends_with(".json"),
                String::from_utf8(f.data.to_vec()).unwrap()
            );
        }
    }
}
