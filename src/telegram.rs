use std::{collections::HashSet, sync::Arc};

use teloxide::utils::command::ParseError;
use teloxide::{
    RequestError,
    dispatching::{DefaultKey, DpHandlerDescription, dialogue::GetChatId},
    error_handlers::ErrorHandler,
    prelude::*,
    sugar::request::RequestLinkPreviewExt,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyParameters, Update, UserId},
    utils::{command::BotCommands, html, markdown},
};

use crate::{ai::Workers, d1::D1, en, google, jp, kotobakku, weblio};
use futures::future::BoxFuture;

#[derive(BotCommands, PartialEq, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "jp -> cn")]
    JPCN(String),
    #[command(description = "cn -> jp")]
    CNJP(String),
    #[command(description = "en <-> cn")]
    EN(String),
    #[command(description = "weblio")]
    Weblio(String),
    #[command(description = "コトバック")]
    Ktbk(String),
    #[command(description = "gemma3 12b it")]
    Gemma(String),
    #[command(description = "llama4 scout 17b 16e instruct")]
    Llama4(String),
    #[command(description = "get current user id")]
    UserID,
    #[command(
        description = "google translate, eg: /gg en_ja hello, /gg ja hello",
        parse_with = split_command
    )]
    GG(String, String, String),
    #[command(
        description = "google translate, eg: /cfai en_ja hello, /cfai ja hello",
        parse_with = split_command
    )]
    CFAI(String, String, String),
    #[command(description = "get a random word from d1 database")]
    Random,
    #[command(description = "save a message by word")]
    Save(String),
}

fn split_command(input: String) -> Result<(String, String, String), ParseError> {
    let mut parts = input.trim().splitn(2, ' ');
    let from_to = parts
        .next()
        .ok_or(ParseError::UnknownCommand("arg1 is not exist".to_string()))?
        .to_string();

    let args = from_to.split("_").collect::<Vec<_>>();
    let mut target = from_to.as_str();
    let mut src = "";

    if args.len() > 1 {
        src = args[0];
        target = args[1];
    }

    let text = parts
        .next()
        .ok_or(ParseError::UnknownCommand("arg2 is not exist".to_string()))?
        .to_string();
    Ok((src.to_string(), target.to_string(), text))
}

pub async fn run_bot(run_opt: RunOpt) -> Dispatcher<Bot, RequestError, DefaultKey> {
    let bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands())
        .send()
        .await
        .unwrap();

    let handler = handler();

    let deps = dptree::deps![run_opt];

    let dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(deps)
        .enable_ctrlc_handler()
        .error_handler(Arc::new(TErrorHandler {}))
        .build();

    return dispatcher;
}

pub struct TErrorHandler {}

impl ErrorHandler<RequestError> for TErrorHandler {
    fn handle_error(self: std::sync::Arc<Self>, error: RequestError) -> BoxFuture<'static, ()> {
        println!("error: {}", error);
        Box::pin(async move {})
    }
}

pub fn handler() -> Handler<'static, Result<(), RequestError>, DpHandlerDescription> {
    return dptree::entry()
        .branch(
            Update::filter_message()
                .branch(dptree::entry().filter_command::<Command>().endpoint(answer)),
        )
        .branch(
            Update::filter_edited_message()
                .branch(dptree::entry().filter_command::<Command>().endpoint(answer)),
        )
        .branch(
            Update::filter_callback_query().branch(
                dptree::entry()
                    .filter_map(|v: CallbackQuery| {
                        v.data
                            .and_then(|x: String| -> Option<CallbackQueryCommand> {
                                match CallbackQueryCommand::parse(&x, "") {
                                    Ok(v) => Some(v),
                                    Err(_) => None,
                                }
                            })
                    })
                    .endpoint(callback_query),
            ),
        );
}

#[derive(Clone)]
pub struct RunOpt {
    pub allow_users: HashSet<UserId>,
    pub d1: D1,
    pub workers_ai: Workers,
}

pub async fn answer(
    mut opt: RunOpt,
    bot: teloxide::prelude::Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), RequestError> {
    let from_user = match &msg.from {
        None => return Ok(()),
        Some(v) => v.id,
    };

    println!("new request from: {}, cmd: {:?}", from_user, cmd);

    if !opt.allow_users.contains(&from_user) {
        println!("user not allowed: {}", from_user);
        return Ok(());
    }

    let mut parse_mode = teloxide::types::ParseMode::MarkdownV2;

    let (word, reply) = match cmd {
        Command::CNJP(word) => match jp::get(word.as_str(), "cj").await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (
                word,
                vec_string_markdown_escape(v.iter().map(|x| x.markdown()).collect()),
            ),
        },
        Command::EN(word) => match en::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (
                word,
                vec_string_markdown_escape(v.iter().map(|x| x.markdown()).collect()),
            ),
        },
        Command::JPCN(word) => match jp::get(word.as_str(), "jc").await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (
                word,
                vec_string_markdown_escape(v.iter().map(|x| x.markdown()).collect()),
            ),
        },
        Command::Ktbk(word) => match kotobakku::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => {
                let reply = vec_string_markdown_escape(v.clone());
                if reply.len() > 4096 {
                    (word, markdown::escape(&v[0].clone()))
                } else {
                    (word, reply)
                }
            }
        },
        Command::Weblio(word) => match weblio::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => {
                let reply = vec_string_markdown_escape(v.clone());
                if reply.len() > 4096 {
                    (word, markdown::escape(&v[0].clone()))
                } else {
                    (word, reply)
                }
            }
        },
        Command::UserID => (
            "".to_string(),
            format!("your id is: {}", from_user).to_string(),
        ),
        Command::GG(from, to, text) => match google::translate(&text, &from, &to).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (
                text,
                markdown::escape(google::merge_translation(v).as_str()),
            ),
        },
        Command::CFAI(from, to, text) => {
            match opt.workers_ai.m2m100_1_2b(&text, &from, &to).await {
                Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
                Ok(v) => (text, markdown::escape(v.as_str())),
            }
        }
        Command::Gemma(v) => match opt.workers_ai.gemma3_12b(v.clone()).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(x) => (v, markdown::escape(x.as_str())),
        },
        Command::Llama4(v) => match opt
            .workers_ai
            .llama4_scout_17b_16e_instruct(v.clone())
            .await
        {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(x) => (v, markdown::escape(x.as_str())),
        },
        Command::Save(v) => {
            if v.is_empty() {
                ("".to_string(), "empty word".to_string())
            } else if msg.reply_to_message().is_none() {
                ("".to_string(), "explain message is empty".to_string())
            } else {
                let reply_to = msg.reply_to_message().unwrap();
                if reply_to.text().is_none() || reply_to.text().unwrap().is_empty() {
                    ("".to_string(), "explain is empty".to_string())
                } else {
                    let reply_to_text = reply_to.text().unwrap().trim();

                    match opt.d1.save_word(v.clone(), reply_to_text.to_string()).await {
                        Err(e) => ("".to_string(), e.to_string()),
                        Ok(_) => (
                            v.clone(),
                            format!("save <b>{}</b> to d1 database", html::escape(v.as_str())),
                        ),
                    }
                }
            }
        }
        Command::Random => match opt.d1.random_word().await {
            Err(e) => ("".to_string(), e.to_string()),
            Ok(v) => {
                parse_mode = teloxide::types::ParseMode::Html;
                (
                    v.word.clone(),
                    format!(
                        "<b>{}</b>\n<tg-spoiler><blockquote expandable>{}</blockquote></tg-spoiler>",
                        html::escape(v.word.as_str()),
                        html::escape(v.explain.as_str())
                    ),
                )
            }
        },
    };

    for v in split_message(&reply, 4096) {
        let mut req = bot
            .send_message(msg.chat.id, v)
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(parse_mode)
            .disable_link_preview(true);

        if !word.is_empty() {
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("🗑️", "/delete"),
                InlineKeyboardButton::callback(
                    "💾",
                    format!(
                        "/save {}",
                        if word.len() < 58 {
                            word.clone()
                        } else {
                            "".to_string()
                        }
                    ),
                ),
            ]]);
            req = req.reply_markup(keyboard);
        }

        req.await?;
    }

    Ok(())
}

fn split_message(text: &str, max_len: usize) -> Vec<String> {
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

#[derive(BotCommands, PartialEq, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:",
    parse_with = "split",
    command_separator = "_"
)]
pub enum CallbackQueryCommand {
    #[command(description = "delete current message")]
    Delete,
    #[command(description = "save word to d1")]
    Save(String),
    #[command(description = "remove word from d1")]
    Remove(String),
}

pub async fn callback_query(
    opt: RunOpt,
    bot: teloxide::prelude::Bot,
    msg: CallbackQuery,
    command: CallbackQueryCommand,
) -> Result<(), RequestError> {
    let from_user = msg.from.id.clone();

    println!("new request from: {}, cmd: {:?}", from_user, command);

    if !opt.allow_users.contains(&from_user) {
        println!("user not allowed: {}", from_user);
        return Ok(());
    }

    match command {
        CallbackQueryCommand::Delete => {
            let chat_id = match msg.chat_id() {
                None => return Ok(()),
                Some(v) => v,
            };

            let msg_id = match msg.message {
                None => return Ok(()),
                Some(v) => v.id(),
            };

            bot.delete_message(chat_id, msg_id).await?;
        }

        CallbackQueryCommand::Save(v) => {
            let msg = match get_callback_message(msg) {
                None => return Ok(()),
                Some(v) => v,
            };

            let explain = match msg.text() {
                None => return Ok(()),
                Some(v) => v.to_string(),
            };

            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("🗑️", "/delete"),
                InlineKeyboardButton::callback("❌", format!("/remove {}", v)),
            ]]);

            match opt.d1.save_word(v, explain).await {
                Err(e) => {
                    println!("save word failed: {}", e);
                    return Ok(());
                }
                _ => {}
            }

            bot.edit_message_reply_markup(msg.chat_id().unwrap(), msg.id)
                .reply_markup(keyboard)
                .await?;
        }
        CallbackQueryCommand::Remove(v) => {
            let msg = match get_callback_message(msg) {
                None => return Ok(()),
                Some(v) => v,
            };

            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("🗑️", "/delete"),
                InlineKeyboardButton::callback("💾", format!("/save {}", v)),
            ]]);

            match opt.d1.delete_word(v).await {
                Err(e) => {
                    println!("delete word failed: {}", e);
                    return Ok(());
                }
                _ => {}
            }

            bot.edit_message_reply_markup(msg.chat_id().unwrap(), msg.id)
                .reply_markup(keyboard)
                .await?;
        }
    };

    Ok(())
}

fn get_callback_message(c: CallbackQuery) -> Option<Message> {
    Some(c.message?.regular_message()?.clone())
}

pub fn vec_string_markdown_escape(v: Vec<String>) -> String {
    let mut s = String::new();
    for i in v {
        s.push_str(markdown::escape(i.as_str()).as_str());
        s.push_str("\n");
    }
    s
}
