use frankenstein::types::{CallbackQuery, Message, MessageEntityType};
use frankenstein::updates::{Update, UpdateContent};
use async_trait::async_trait;
use log::info;
use crate::utils::{get_utf16_slice, get_utf16_slice_rest};

#[async_trait(?Send)]
pub trait TelegramBot {
    type Error: std::fmt::Debug;

    async fn handle_command(&self, msg: Message, cmd: &str, args: &str) -> Result<(), Self::Error>;

    async fn handle_callback(
        &self,
        query: CallbackQuery,
        cmd: &str,
        args: &str,
    ) -> Result<(), Self::Error>;

    async fn handle_message(&self, _msg: Message) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub async fn process_update<B: TelegramBot>(bot: &B, update: Update) -> Result<(), B::Error> {
    match update.content {
        UpdateContent::Message(msg) | UpdateContent::EditedMessage(msg) => {
            let command_info = msg.entities.as_ref().and_then(|entities| {
                entities
                    .iter()
                    .find(|e| e.type_field == MessageEntityType::BotCommand)
                    .map(|e| (e.offset, e.length))
            });

            if let Some((offset, length)) = command_info {
                let txt = msg.text.clone().unwrap_or_default();
                let command = get_utf16_slice(&txt, offset as usize, length as usize);
                let argument =
                    get_utf16_slice_rest(&txt, offset as usize + length as usize)
                        .trim()
                        .to_string();

                info!("message command: {:?}, argument: {:?}", command, argument);
                bot.handle_command(*msg, &command, &argument).await
            } else {
                bot.handle_message(*msg).await
            }
        }
        UpdateContent::CallbackQuery(query) => {
            let data = query.data.clone().unwrap_or_default();
            let mut parts = data.splitn(2, ' ');
            let command = parts.next().unwrap_or("");
            let argument = parts.next().unwrap_or("");

            info!(
                "callback query command: {:?}, argument: {:?}",
                command, argument
            );
            bot.handle_callback(*query, command, argument).await
        }
        _ => Ok(()),
    }
}
