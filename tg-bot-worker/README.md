# tg-bot-worker

A generic, runtime-agnostic Telegram bot framework in Rust, designed specifically to support single-threaded environments like Cloudflare Workers.

## Features

- **Runtime-Agnostic:** Uses `async-trait` (`?Send`) for compatibility with generic async executors, such as Tokio and single-threaded runtimes like WebAssembly / Cloudflare Workers.
- **Robust UTF-16 Handling:** Designed to correctly slice strings via Telegram's UTF-16 based `offset` and `length` properties, handling multi-byte characters (like CJK texts and emojis) flawlessly without panicking or creating invalid strings.
- **Helper Utilities:** Provides optimized functions to slice strings, split excessively long strings securely, and escape characters for Telegram's MarkdownV2 and HTML modes.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tg-bot-worker = "0.1"
```

If you are using it in a regular Tokio environment, you can use the same code as in a single-threaded runtime.

## Usage

Define a struct to implement the `TelegramBot` trait. The framework provides a default `handle_update` method which parses the incoming `frankenstein::updates::Update` and routes it to specific methods like `handle_command` and `handle_callback`.

```rust
use async_trait::async_trait;
use frankenstein::types::{Message, CallbackQuery};
use tg_bot_worker::TelegramBot;

pub struct MyBot;

#[async_trait(?Send)]
impl TelegramBot for MyBot {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    async fn handle_command(
        &self,
        msg: Box<Message>,
        command: &str,
        argument: &str,
        quote: &str,
    ) -> Result<(), Self::Error> {
        println!("Received command: {}", command);
        println!("With arguments: {}", argument);
        Ok(())
    }

    async fn handle_callback(
        &self,
        call_query: Box<CallbackQuery>,
        command: &str,
        argument: &str,
    ) -> Result<(), Self::Error> {
        println!("Received callback query: {}", command);
        Ok(())
    }
}
```

## Example

For a complete working example of setting up a simple bot, check the `examples/basic_bot.rs` file. You can run it locally with:

```bash
cargo run --example basic_bot
```

## Utilities

The framework includes a few helpful utility methods under `tg_bot_worker::utils`:

- `markdown_escape(text: &str) -> Cow<str>`
- `html_escape(text: &str) -> Cow<str>`
- `split_message(text: &str, max_len: usize) -> Vec<String>`
- `vec_string_markdown_escape(v: &[String]) -> String`
- `utf16_slice(text: &str, start_utf16: usize, len_utf16: usize) -> Option<&str>`
- `utf16_slice_from(text: &str, start_utf16: usize) -> Option<&str>`
