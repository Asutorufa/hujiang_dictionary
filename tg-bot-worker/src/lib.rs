use async_trait::async_trait;
use frankenstein::types::{CallbackQuery, Message, MessageEntityType};
use frankenstein::updates::{Update, UpdateContent};
use log::{error, warn};

pub mod macros;
pub mod utils;

#[async_trait(?Send)]
pub trait TelegramBot {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn handle_command(
        &self,
        msg: Box<Message>,
        command: &str,
        argument: &str,
        quote: &str,
    ) -> Result<(), Self::Error>;

    async fn handle_callback(
        &self,
        call_query: Box<CallbackQuery>,
        command: &str,
        argument: &str,
    ) -> Result<(), Self::Error>;

    async fn handle_message(&self, msg: Box<Message>) -> Result<(), Self::Error> {
        let _ = msg;
        Ok(())
    }

    async fn handle_update(&self, update: Update) -> Result<(), Self::Error> {
        match update.content {
            UpdateContent::Message(msg) | UpdateContent::EditedMessage(msg) => {
                let entity = match msg.entities.as_ref() {
                    Some(v)
                        if !v.is_empty() && v[0].type_field == MessageEntityType::BotCommand =>
                    {
                        &v[0]
                    }
                    _ => return self.handle_message(msg).await,
                };

                let txt = match msg.text.as_ref() {
                    Some(v) => v,
                    None => return self.handle_message(msg).await,
                };

                let quote_or_reply_message = msg
                    .quote
                    .as_ref()
                    .map(|v| v.text.as_str())
                    .or_else(|| msg.reply_to_message.as_ref().and_then(|v| v.text.as_deref()))
                    .unwrap_or("");

                let command =
                    match utils::utf16_slice(txt, entity.offset as usize, entity.length as usize) {
                        Some(v) => v.to_string(),
                        None => {
                            error!("Failed to slice command out of text");
                            return self.handle_message(msg).await;
                        }
                    };

                let argument = match utils::utf16_slice_from(
                    txt,
                    entity.offset as usize + entity.length as usize,
                ) {
                    Some(v) => v.trim().to_string(),
                    None => {
                        error!("Failed to slice argument out of text");
                        return self.handle_message(msg).await;
                    }
                };

                let quote_or_reply_message = quote_or_reply_message.to_string();

                self.handle_command(msg, &command, &argument, &quote_or_reply_message)
                    .await
            }
            UpdateContent::CallbackQuery(msg) => {
                let (command, argument) = match msg.data.as_ref() {
                    Some(v) => {
                        let mut data = v.splitn(2, ' ');
                        let command = data.next().unwrap_or("");
                        let argument = data.next().unwrap_or("");
                        (command.to_string(), argument.to_string())
                    }
                    None => {
                        warn!("Callback query has no data");
                        return Ok(());
                    }
                };

                self.handle_callback(msg, command.as_str(), argument.as_str())
                    .await
            }
            _ => Ok(()),
        }
    }
}
