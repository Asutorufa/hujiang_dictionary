# tg-bot-worker

A lightweight, generic framework for building Telegram bots, designed with Cloudflare Workers (and other WASM environments) in mind.

It abstracts away the boilerplate of parsing updates, routing commands, and handling callbacks, while remaining agnostic to the underlying HTTP client or runtime.

## Features

- **Runtime Agnostic**: Works on Cloudflare Workers (`worker-rs`), native Rust, or any other environment.
- **Async Trait**: Uses `async_trait` with `?Send` support, making it compatible with single-threaded runtimes like Cloudflare Workers.
- **Command Parsing**: Automatically parses `/command arguments` from messages.
- **Callback Queries**: Handles callback queries with a simple `cmd args` convention.
- **Safe Utilities**: Includes helpers for safe UTF-16 string slicing (critical for correct Telegram entity offsets) and Markdown/HTML escaping.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tg-bot-worker = "0.1"
frankenstein = "0.46" # For types
async-trait = "0.1"
```

## Usage

Implement the `TelegramBot` trait for your bot struct:

```rust
use tg_bot_worker::{TelegramBot, process_update};
use frankenstein::types::{Message, CallbackQuery, Update};
use async_trait::async_trait;

struct MyBot;

#[async_trait(?Send)]
impl TelegramBot for MyBot {
    type Error = std::convert::Infallible;

    async fn handle_command(&self, msg: Message, cmd: &str, args: &str) -> Result<(), Self::Error> {
        println!("Received command: {} with args: {}", cmd, args);
        // logic to send reply...
        Ok(())
    }

    async fn handle_callback(&self, query: CallbackQuery, cmd: &str, args: &str) -> Result<(), Self::Error> {
        println!("Received callback: {} with args: {}", cmd, args);
        Ok(())
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Self::Error> {
        println!("Received plain message: {:?}", msg.text);
        Ok(())
    }
}

// In your request handler (e.g., worker fetch event):
async fn handle_request(update: Update) {
    let bot = MyBot;
    process_update(&bot, update).await.unwrap();
}
```

## Utilities

The crate exports useful utilities in `tg_bot_worker::utils`:
- `markdown_escape(s)`: Escape generic Markdown characters.
- `html_escape(s)`: Escape HTML special characters.
- `split_message(text, limit)`: Split long text into chunks (safe for UTF-8).
- `get_utf16_slice(s, offset, length)`: Safe slicing using Telegram's UTF-16 offsets.

## License

MIT
