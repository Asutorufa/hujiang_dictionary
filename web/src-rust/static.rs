use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "out/"]
pub struct Assets;

#[cfg(test)]
mod test {
    use crate::r#static::Assets;

    #[test]
    fn test() {
        for v in Assets::iter() {
            println!("{}", v);
        }
    }
}
