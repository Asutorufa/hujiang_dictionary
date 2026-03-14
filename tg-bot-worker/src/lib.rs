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
                let command;
                let argument;
                let quote_or_reply_message;

                // create a block to limit the borrow scope of msg
                {
                    let entity = match msg.entities.as_ref() {
                        Some(v)
                            if !v.is_empty()
                                && v[0].type_field == MessageEntityType::BotCommand =>
                        {
                            &v[0]
                        }
                        _ => return self.handle_message(msg).await,
                    };
                    let command_len = entity.length as usize;
                    let command_offset = entity.offset as usize;

                    let txt = match msg.text.as_ref() {
                        Some(v) => v,
                        None => return self.handle_message(msg).await,
                    };

                    quote_or_reply_message = msg
                        .quote
                        .as_ref()
                        .map(|v| v.text.as_str())
                        .or_else(|| {
                            msg.reply_to_message
                                .as_ref()
                                .and_then(|v| v.text.as_deref())
                        })
                        .unwrap_or("")
                        .to_string();

                    command = match utils::utf16_slice(txt, command_offset, command_len) {
                        Some(v) => v.to_string(),
                        None => {
                            error!("Failed to slice command out of text");
                            return self.handle_message(msg).await;
                        }
                    };

                    argument = match utils::utf16_slice_from(txt, command_offset + command_len) {
                        Some(v) => v.trim().to_string(),
                        None => {
                            error!("Failed to slice argument out of text");
                            return self.handle_message(msg).await;
                        }
                    };
                }

                self.handle_command(msg, &command, &argument, &quote_or_reply_message)
                    .await
            }
            UpdateContent::CallbackQuery(msg) => {
                let data = match msg.data.as_ref() {
                    Some(v) => v.clone(),
                    None => {
                        warn!("Callback query has no data");
                        return Ok(());
                    }
                };

                let mut parts = data.splitn(2, ' ');
                let command = parts.next().unwrap_or("");
                let argument = parts.next().unwrap_or("");

                self.handle_callback(msg, command, argument).await
            }
            _ => Ok(()),
        }
    }
}
