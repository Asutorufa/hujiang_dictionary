use async_trait::async_trait;
use frankenstein::types::{CallbackQuery, Message};
use frankenstein::updates::Update;
use std::fmt;
use tg_bot_worker::TelegramBot;

pub struct SimpleBot;

#[derive(Debug)]
pub struct BotError(String);

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BotError {}

#[async_trait(?Send)]
impl TelegramBot for SimpleBot {
    type Error = BotError;

    async fn handle_command(
        &self,
        msg: Box<Message>,
        command: &str,
        argument: &str,
        quote: &str,
    ) -> Result<(), Self::Error> {
        println!("Received command: {}", command);
        println!("Arguments: {}", argument);
        println!("Quoted/Replied: {}", quote);

        println!("Message from: {:?}", msg.from.map(|u| u.username));
        Ok(())
    }

    async fn handle_callback(
        &self,
        _call_query: Box<CallbackQuery>,
        command: &str,
        argument: &str,
    ) -> Result<(), Self::Error> {
        println!("Received callback command: {}", command);
        println!("Callback args: {}", argument);
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = SimpleBot;

    // In a real application, you would parse the update from a JSON HTTP request
    // e.g. from an AWS API Gateway or a Cloudflare Worker request body.
    let simulated_update_json = r#"{
        "update_id": 123456789,
        "message": {
            "message_id": 1,
            "from": {
                "id": 123,
                "is_bot": false,
                "first_name": "Test",
                "username": "testuser"
            },
            "chat": {
                "id": 123,
                "first_name": "Test",
                "username": "testuser",
                "type": "private"
            },
            "date": 1620000000,
            "text": "/hello world",
            "entities": [
                {
                    "offset": 0,
                    "length": 6,
                    "type": "bot_command"
                }
            ]
        }
    }"#;

    let update: Update = serde_json::from_str(simulated_update_json)?;

    println!("Passing update to bot handler...");
    bot.handle_update(update).await?;

    Ok(())
}
