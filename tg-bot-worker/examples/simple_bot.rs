use tg_bot_worker::{TelegramBot, process_update};
use frankenstein::types::{Message, CallbackQuery};
use frankenstein::updates::Update;
use async_trait::async_trait;
use std::convert::Infallible;

struct MyBot;

#[async_trait(?Send)]
impl TelegramBot for MyBot {
    type Error = Infallible;

    async fn handle_command(&self, _msg: Message, cmd: &str, args: &str) -> Result<(), Self::Error> {
        println!("Received command: {} with args: {}", cmd, args);
        // In a real bot, you would send a reply using frankenstein::Bot
        Ok(())
    }

    async fn handle_callback(&self, _query: CallbackQuery, cmd: &str, args: &str) -> Result<(), Self::Error> {
        println!("Received callback: {} with args: {}", cmd, args);
        Ok(())
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Self::Error> {
        if let Some(text) = msg.text {
            println!("Received plain message: {}", text);
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let bot = MyBot;

    // Simulate a command update using JSON
    let json = r#"
    {
        "update_id": 1,
        "message": {
            "message_id": 123,
            "date": 1600000000,
            "chat": {
                "id": 12345,
                "type": "private",
                "username": "user",
                "first_name": "User"
            },
            "from": {
                "id": 12345,
                "is_bot": false,
                "first_name": "User"
            },
            "text": "/hello world",
            "entities": [
                {
                    "offset": 0,
                    "length": 6,
                    "type": "bot_command"
                }
            ]
        }
    }
    "#;

    // Use fully qualified path for serde_json which is a dev-dependency
    let update: Update = serde_json::from_str(json).expect("Failed to parse JSON");

    if let Err(e) = process_update(&bot, update).await {
        println!("Error processing update: {:?}", e);
    }
}
