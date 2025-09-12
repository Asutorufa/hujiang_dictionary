use log::{Level, Log, Metadata, Record, SetLoggerError};
use web_sys::console;

static LOGGER: WebConsoleLogger = WebConsoleLogger {};

struct WebConsoleLogger {}

impl Log for WebConsoleLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        match record.module_path().unwrap_or("") {
            "html5ever::serialize" => return,
            _ => {}
        }

        if !self.enabled(record.metadata()) {
            return;
        }

        let console_log = match record.level() {
            Level::Error => console::error_1,
            Level::Warn => console::warn_1,
            Level::Info => console::info_1,
            Level::Debug => console::log_1,
            Level::Trace => console::debug_1,
        };

        let module_path = record.module_path().unwrap_or("");
        // let file = record.file().unwrap_or("");
        // let line = record.line().unwrap_or(0);

        console_log(&format!("[{}] {}", module_path, record.args()).into());
    }

    fn flush(&self) {}
}

#[inline]
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}
