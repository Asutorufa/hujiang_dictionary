use crate::{ai::AI, d1::DB, opts::RunOpt};
use core::fmt;
use frankenstein::AsyncTelegramApi;
use frankenstein::client_reqwest::Bot;
use frankenstein::methods::{SendMessageParams, SetMyCommandsParams, SetWebhookParams};
use frankenstein::types::{
    ChatId, LinkPreviewOptions, MaybeInaccessibleMessage, MessageEntityType,
};
use frankenstein::updates::UpdateContent;
use hjdict::{en, google, jp, kotobakku, weblio};
use log::*;
use std::sync::Arc;

#[derive(Debug)]
pub enum Command {
    JPCN(String),
    CNJP(String),
    EN(String),
    Weblio(String),
    Ktbk(String),
    Gemma(String),
    Llama4(String),
    UserID,
    GG(Option<String>, String, String),
    CFAI(Option<String>, String, String),
    Random,
    Save(String),
}

pub fn bot_commands() -> Vec<frankenstein::types::BotCommand> {
    vec![
        frankenstein::types::BotCommand {
            command: "jpcn".to_string(),
            description: "jp -> cn".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "cnjp".to_string(),
            description: "cn -> jp".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "en".to_string(),
            description: "en <-> cn".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "weblio".to_string(),
            description: "weblio".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "ktbk".to_string(),
            description: "コトバック".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "gemma".to_string(),
            description: "gemma3 12b it".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "llama4".to_string(),
            description: "llama4 scout 17b 16e instruct".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "userid".to_string(),
            description: "get current user id".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "gg".to_string(),
            description: "google translate, eg: /gg en_ja hello, /gg ja hello".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "cfai".to_string(),
            description: "google translate, eg: /cfai en_ja hello, /cfai ja hello".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "random".to_string(),
            description: "get a random word from d1 database".to_string(),
        },
        frankenstein::types::BotCommand {
            command: "save".to_string(),
            description: "save a message by word".to_string(),
        },
    ]
}

pub fn split_message(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for c in text.chars() {
        if current.len() + c.len_utf8() > max_len {
            chunks.push(current);
            current = String::new();
        }
        current.push(c);
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[derive(Debug)]
pub enum CallbackQueryCommand {
    Delete,
    Save(String),
    Remove(String),
}

pub(super) const MARKDOWN_ESCAPE_CHARS: [char; 19] = [
    '\\', '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
];

pub fn markdown_escape(s: &str) -> String {
    s.chars().fold(String::with_capacity(s.len()), |mut s, c| {
        if MARKDOWN_ESCAPE_CHARS.contains(&c) {
            s.push('\\');
        }
        s.push(c);
        s
    })
}

pub fn html_escape(s: &str) -> String {
    s.chars().fold(String::with_capacity(s.len()), |mut s, c| {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            c => s.push(c),
        }
        s
    })
}

pub fn vec_string_markdown_escape(v: &Vec<String>) -> String {
    let mut s = String::new();
    for i in v {
        s.push_str(markdown_escape(i.as_str()).as_str());
        s.push_str("\n");
    }
    s
}

pub async fn handle<T: DB, T2: AI>(
    opt: Arc<RunOpt<T, T2>>,
    update: frankenstein::updates::Update,
) -> Result<(), Error> {
    match update.content {
        UpdateContent::Message(msg) | UpdateContent::EditedMessage(msg) => {
            let entity = match msg.entities.as_ref() {
                Some(v) if !v.is_empty() && v[0].type_field == MessageEntityType::BotCommand => {
                    &v[0]
                }
                _ => return Err(Error("no command entity".to_string())),
            };

            let txt = match msg.text.as_ref() {
                Some(v) => v,
                None => return Err(Error("no text".to_string())),
            };

            let command =
                &txt[entity.offset as usize..entity.offset as usize + entity.length as usize];

            let argument = txt[entity.offset as usize + entity.length as usize..].trim();

            let (cmd, text) = parse_command(command, argument)?;

            info!("message command: {:?}, argument: {:?}", cmd, text);

            answer(opt, msg, cmd).await?;
        }
        UpdateContent::CallbackQuery(msg) => {
            let (command, argument) = match msg.data.as_ref() {
                Some(v) => {
                    let mut data = v.splitn(2, ' ');
                    let command = data.next().unwrap_or("");
                    let argument = data.next().unwrap_or("");
                    (command.to_string(), argument.to_string())
                }
                None => return Err(Error("no data".to_string())),
            };

            let (cmd, text) = parse_callback_query_command(command.as_str(), argument.as_str())?;

            info!("callback query command: {:?}, argument: {:?}", cmd, text);

            callback_query(opt, msg, cmd).await?;
        }
        _ => return Err(Error("not message".to_string())),
    };

    Ok(())
}

#[derive(Debug)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<frankenstein::Error> for Error {
    fn from(value: frankenstein::Error) -> Self {
        Error(value.to_string())
    }
}

pub fn parse_callback_query_command(
    command: &str,
    argument: &str,
) -> Result<(CallbackQueryCommand, Option<String>), Error> {
    let command = command.trim_start_matches("/");
    match command {
        "delete" => Ok((CallbackQueryCommand::Delete, None)),
        "save" => Ok((CallbackQueryCommand::Save(argument.to_string()), None)),
        "remove" => Ok((CallbackQueryCommand::Remove(argument.to_string()), None)),
        _ => Err(Error("not implemented".to_string())),
    }
}

pub fn parse_command(command: &str, argument: &str) -> Result<(Command, Option<String>), Error> {
    let command = command
        .trim_start_matches("/")
        .split('@')
        .next()
        .unwrap_or("");

    if command.is_empty() {
        return Err(Error("command is empty".to_string()));
    }

    match command {
        "cnjp" => Ok((Command::CNJP(argument.to_string()), None)),
        "jpcn" => Ok((Command::JPCN(argument.to_string()), None)),
        "en" => Ok((Command::EN(argument.to_string()), None)),
        "weblio" => Ok((Command::Weblio(argument.to_string()), None)),
        "ktbk" => Ok((Command::Ktbk(argument.to_string()), None)),
        "gemma" => Ok((Command::Gemma(argument.to_string()), None)),
        "random" => Ok((Command::Random, None)),
        "llama4" => Ok((Command::Llama4(argument.to_string()), None)),
        "save" => Ok((Command::Save(argument.to_string()), None)),
        "gg" | "cfai" => {
            let mut parts = argument.splitn(2, ' ');
            let first = parts.next().unwrap_or("");
            let rest = parts.next().unwrap_or("").to_string();

            let args = first.split("_").collect::<Vec<_>>();
            let mut target = first.to_string();
            let mut src: Option<String> = None;

            if args.len() > 1 {
                src = if args[0].is_empty() {
                    None
                } else {
                    Some(args[0].to_string())
                };
                target = args[1].to_string();
            }

            if command == "cfai" {
                Ok((Command::CFAI(src, target, rest), None))
            } else {
                Ok((Command::GG(src, target, rest), None))
            }
        }
        "userid" => Ok((Command::UserID, None)),

        _ => Err(Error("not implemented".to_string())),
    }
}

pub async fn answer<T: DB, T2: AI>(
    opt: Arc<RunOpt<T, T2>>,
    msg: Box<frankenstein::types::Message>,
    cmd: Command,
) -> Result<(), frankenstein::Error> {
    let from_user = match &msg.from {
        None => return Ok(()),
        Some(v) => v.id,
    };

    info!("new request from: {}, cmd: {:?}", from_user, cmd);

    if !opt.allow_users.contains(&(from_user as i64)) {
        warn!("user not allowed: {}", from_user);
        return Ok(());
    }

    let mut parse_mode = frankenstein::ParseMode::MarkdownV2;

    let (word, reply) = match cmd {
        Command::CNJP(word) => match jp::get(word.as_str(), "cj").await {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(v) => (
                word,
                vec_string_markdown_escape(&v.iter().map(|x| x.markdown()).collect()),
            ),
        },
        Command::EN(word) => match en::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(v) => (
                word,
                vec_string_markdown_escape(&v.iter().map(|x| x.markdown()).collect()),
            ),
        },
        Command::JPCN(word) => match jp::get(word.as_str(), "jc").await {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(v) => (
                word,
                vec_string_markdown_escape(&v.iter().map(|x| x.markdown()).collect()),
            ),
        },
        Command::Ktbk(word) => match kotobakku::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(v) => {
                let reply = vec_string_markdown_escape(&v);
                if reply.len() > 4096 {
                    (word, markdown_escape(&v[0]))
                } else {
                    (word, reply)
                }
            }
        },
        Command::Weblio(word) => match weblio::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(v) => {
                let reply = vec_string_markdown_escape(&v);
                if reply.len() > 4096 {
                    (word, markdown_escape(&v[0]))
                } else {
                    (word, reply)
                }
            }
        },
        Command::UserID => (
            "".to_string(),
            format!("your id is: {}", from_user).to_string(),
        ),
        Command::GG(from, to, text) => {
            match google::translate(text.as_ref(), from, to.as_ref()).await {
                Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
                Ok(v) => (text, markdown_escape(google::merge_translation(v).as_str())),
            }
        }
        Command::CFAI(from, to, text) => {
            match opt.workers_ai.m2m100_1_2b(text.as_ref(), from, to).await {
                Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
                Ok(v) => (text, markdown_escape(v.as_str())),
            }
        }
        Command::Gemma(v) => match opt.workers_ai.gemma3_12b(format!("{}\n{}", "", v)).await {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(x) => (v, markdown_escape(x.as_str())),
        },
        Command::Llama4(v) => match opt
            .workers_ai
            .llama4_scout_17b_16e_instruct(v.as_ref())
            .await
        {
            Err(e) => ("".to_string(), markdown_escape(e.to_string().as_str())),
            Ok(x) => (v, markdown_escape(x.as_str())),
        },
        Command::Save(v) => {
            if v.is_empty() {
                ("".to_string(), "empty word".to_string())
            } else if msg.reply_to_message.is_none() {
                ("".to_string(), "explain message is empty".to_string())
            } else {
                let reply_to = msg.reply_to_message.unwrap();
                match reply_to.text.as_ref() {
                    None => ("".to_string(), "explain is empty".to_string()),
                    Some(v) => {
                        let reply_to_text = v.trim();
                        match opt.d1.save_word(v.as_ref(), reply_to_text.as_ref()).await {
                            Err(e) => ("".to_string(), e.to_string()),
                            Ok(_) => (
                                v.to_owned(),
                                format!("save <b>{}</b> to d1 database", html_escape(v.as_str())),
                            ),
                        }
                    }
                }
            }
        }
        Command::Random => match opt.d1.random_word().await.as_ref() {
            Err(e) => ("".to_string(), e.to_string()),
            Ok(v) => {
                parse_mode = frankenstein::ParseMode::Html;
                (
                    v.word.to_owned(),
                    format!(
                        "<b>{}</b>\n<tg-spoiler><blockquote expandable>{}</blockquote></tg-spoiler>",
                        html_escape(v.word.as_str()),
                        html_escape(v.explain.as_str())
                    ),
                )
            }
        },
    };

    for v in split_message(&reply, 4096) {
        if v.is_empty() {
            continue;
        }

        let mut req = SendMessageParams::builder()
            .chat_id(msg.chat.id)
            .text(v)
            .reply_parameters(
                frankenstein::types::ReplyParameters::builder()
                    .message_id(msg.message_id)
                    .build(),
            )
            .link_preview_options(LinkPreviewOptions::DISABLED)
            .parse_mode(parse_mode)
            .build();

        req.reply_markup = if word.is_empty() {
            None
        } else {
            Some(frankenstein::types::ReplyMarkup::InlineKeyboardMarkup(
                frankenstein::types::InlineKeyboardMarkup::builder()
                    .inline_keyboard(vec![vec![
                        frankenstein::types::InlineKeyboardButton::builder()
                            .text("🗑️")
                            .callback_data("/delete")
                            .build(),
                        frankenstein::types::InlineKeyboardButton::builder()
                            .text("💾")
                            .callback_data(format!(
                                "/save {}",
                                if word.len() < 58 { word.as_ref() } else { "" }
                            ))
                            .build(),
                    ]])
                    .build(),
            ))
        };

        opt.bot.send_message(&req).await?;
    }

    Ok(())
}

pub async fn callback_query<T: DB, T2: AI>(
    opt: Arc<RunOpt<T, T2>>,
    call_query: Box<frankenstein::types::CallbackQuery>,
    command: CallbackQueryCommand,
) -> Result<(), Error> {
    let from_user = call_query.from.id;

    info!("new request from: {}, cmd: {:?}", from_user, command);

    if !opt.allow_users.contains(&(from_user as i64)) {
        warn!("user not allowed: {}", from_user);
        return Ok(());
    }

    let (chat_id, msg_id, text) = match call_query.message {
        None => return Ok(()),
        Some(v) => match v {
            MaybeInaccessibleMessage::Message(v) => (v.chat.id, v.message_id, v.text),
            MaybeInaccessibleMessage::InaccessibleMessage(v) => (v.chat.id, v.message_id, None),
        },
    };

    match command {
        CallbackQueryCommand::Delete => {
            let req = frankenstein::methods::DeleteMessageParams::builder()
                .chat_id(chat_id)
                .message_id(msg_id)
                .build();

            opt.bot.delete_message(&req).await?;
        }

        CallbackQueryCommand::Save(v) => {
            let explain = match text {
                None => return Ok(()),
                Some(v) => v.to_string(),
            };

            match opt.d1.save_word(v.as_ref(), explain.as_ref()).await {
                Err(e) => {
                    error!("save word failed: {}", e);
                    return Ok(());
                }
                _ => {}
            }

            let req = frankenstein::methods::EditMessageReplyMarkupParams::builder()
                .chat_id(chat_id)
                .message_id(msg_id)
                .reply_markup(
                    frankenstein::types::InlineKeyboardMarkup::builder()
                        .inline_keyboard(vec![vec![
                            frankenstein::types::InlineKeyboardButton::builder()
                                .text("🗑️")
                                .callback_data("/delete")
                                .build(),
                            frankenstein::types::InlineKeyboardButton::builder()
                                .text("❌")
                                .callback_data(format!("/remove {}", v))
                                .build(),
                        ]])
                        .build(),
                )
                .build();

            opt.bot.edit_message_reply_markup(&req).await?;
        }
        CallbackQueryCommand::Remove(v) => {
            match opt.d1.delete_word(v.as_ref()).await {
                Err(e) => {
                    error!("delete word failed: {}", e);
                    return Ok(());
                }
                _ => {}
            }

            let req = frankenstein::methods::EditMessageReplyMarkupParams::builder()
                .chat_id(chat_id)
                .message_id(msg_id)
                .reply_markup(
                    frankenstein::types::InlineKeyboardMarkup::builder()
                        .inline_keyboard(vec![vec![
                            frankenstein::types::InlineKeyboardButton::builder()
                                .text("🗑️")
                                .callback_data("/delete")
                                .build(),
                            frankenstein::types::InlineKeyboardButton::builder()
                                .text("💾")
                                .callback_data(format!("/save {}", v))
                                .build(),
                        ]])
                        .build(),
                )
                .build();

            opt.bot.edit_message_reply_markup(&req).await?;
        }
    };

    Ok(())
}

pub async fn send_random_word<T: DB, T2: AI>(
    opt: Arc<RunOpt<T, T2>>,
) -> Result<(), frankenstein::Error> {
    let reply = match opt.d1.random_word().await {
        Err(e) => e.to_string(),
        Ok(v) => {
            format!(
                "<b>{}</b>\n<tg-spoiler><blockquote expandable>{}</blockquote></tg-spoiler>",
                html_escape(v.word.as_str()),
                html_escape(v.explain.as_str())
            )
        }
    };

    opt.bot
        .send_message(
            &SendMessageParams::builder()
                .chat_id(frankenstein::types::ChatId::Integer(opt.matainer as i64))
                .text(reply)
                .parse_mode(frankenstein::ParseMode::Html)
                .link_preview_options(frankenstein::types::LinkPreviewOptions::DISABLED)
                .build(),
        )
        .await?;

    Ok(())
}

pub async fn set_webhook(bot: &Bot, url: &str, matainer: i64) -> Result<(), Error> {
    info!("Registering webhook: {}", url);

    bot.set_my_commands(
        &SetMyCommandsParams::builder()
            .commands(bot_commands())
            .build(),
    )
    .await?;

    bot.set_webhook(&SetWebhookParams::builder().url(url).build())
        .await?;

    bot.send_message(
        &SendMessageParams::builder()
            .chat_id(ChatId::Integer(matainer))
            .text(format!("register webhook to {} successful", url))
            .link_preview_options(LinkPreviewOptions::DISABLED)
            .build(),
    )
    .await?;

    Ok(())
}
