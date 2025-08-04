use std::collections::HashSet;

use teloxide::{
    RequestError,
    dispatching::{DefaultKey, DpHandlerDescription, dialogue::GetChatId},
    prelude::*,
    sugar::request::RequestLinkPreviewExt,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyParameters, Update, UserId},
    utils::{command::BotCommands, html, markdown},
};

use crate::{ai::Workers, d1::D1, google, jp, kotobakku, weblio};

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
    #[command(description = "weblio")]
    Weblio(String),
    #[command(description = "コトバック")]
    Ktbk(String),
    #[command(description = "get current user id")]
    UserID,
    #[command(
        description = "google translate, eg: /gg en_ja hello, /gg ja hello",
        parse_with = "split"
    )]
    GG(String, String),
    #[command(description = "get a random word from d1 database")]
    Random,
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
        .build();

    return dispatcher;
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
    opt: RunOpt,
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
            Ok(v) => (word, markdown::escape(format!("{:?}", v).as_str())),
        },
        Command::JPCN(word) => match jp::get(word.as_str(), "jc").await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (word, markdown::escape(format!("{:?}", v).as_str())),
        },
        Command::Ktbk(word) => match kotobakku::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (word, markdown::escape(format!("{:?}", v).as_str())),
        },
        Command::Weblio(word) => match weblio::get(word.as_str()).await {
            Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
            Ok(v) => (word, markdown::escape(format!("{:?}", v).as_str())),
        },
        Command::UserID => (
            "".to_string(),
            format!("your id is: {}", from_user).to_string(),
        ),
        Command::GG(arg, text) => {
            let args = arg.split("_").collect::<Vec<_>>();
            let mut target = arg.as_str();
            let mut src = "";

            if args.len() > 1 {
                src = args[0];
                target = args[1];
            }

            match google::translate(&text, src, target).await {
                Err(e) => ("".to_string(), markdown::escape(e.to_string().as_str())),
                Ok(v) => (
                    text,
                    markdown::escape(google::merge_translation(v).as_str()),
                ),
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

    let mut req = bot
        .send_message(msg.chat.id, reply)
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(parse_mode);

    if !word.is_empty() {
        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("🗑️", "/delete"),
            InlineKeyboardButton::callback("💾", format!("/save {}", word)),
        ]]);
        req = req.reply_markup(keyboard);
    }

    req.disable_link_preview(true).await?;
    Ok(())
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
