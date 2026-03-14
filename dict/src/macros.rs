macro_rules! sel {
    ($selector:expr) => {{
        static SELECTOR: ::std::sync::OnceLock<::scraper::Selector> = ::std::sync::OnceLock::new();
        SELECTOR.get_or_init(|| ::scraper::Selector::parse($selector).unwrap())
    }};
}
