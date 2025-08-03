use std::collections::HashSet;

use teloxide::{
    RequestError,
    dispatching::{DefaultKey, DpHandlerDescription},
    prelude::*,
    sugar::request::RequestLinkPreviewExt,
    types::{ReplyParameters, Update, UserId},
    utils::{command::BotCommands, html},
};

use crate::{ai::Workers, d1::D1, google, jp};

#[derive(BotCommands, PartialEq, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:",
    parse_with = "split",
    command_separator = "_"
)]
pub enum Command {
    #[command(description = "jp -> cn")]
    JPCN(String),
    #[command(description = "cn -> jp")]
    CNJP(String),
    #[command(description = "get current user id")]
    UserID,
    #[command(description = "google translate, eg: /gg_en_ja hello, /gg_ja hello")]
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

    if !opt.allow_users.contains(&from_user) {
        return Ok(());
    }

    println!("new request from: {}", from_user);

    let mut parse_mode = teloxide::types::ParseMode::MarkdownV2;

    let reply = match cmd {
        Command::CNJP(word) => match jp::get(word.as_str(), "cj").await {
            Err(e) => e.to_string(),
            Ok(v) => format!("{:?}", v),
        },
        Command::JPCN(word) => match jp::get(word.as_str(), "jc").await {
            Err(e) => e.to_string(),
            Ok(v) => format!("{:?}", v),
        },
        Command::UserID => format!("your id is: {}", from_user).to_string(),
        Command::GG(arg, text) => {
            let args = arg.split("_").collect::<Vec<_>>();
            let mut target = arg.as_str();
            let mut src = "";

            if args.len() > 1 {
                src = args[0];
                target = args[1];
            }

            match google::translate(&text, src, target).await {
                Err(e) => e.to_string(),
                Ok(v) => google::merge_translation(v),
            }
        }
        Command::Random => match opt.d1.random_word().await {
            Err(e) => e.to_string(),
            Ok(v) => {
                parse_mode = teloxide::types::ParseMode::Html;
                format!(
                    "<b>{}</b>\n<tg-spoiler><blockquote expandable>{}</blockquote></tg-spoiler>",
                    html::escape(v.word.as_str()),
                    html::escape(v.explain.as_str())
                )
            }
        },
    };

    bot.send_message(msg.chat.id, reply)
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(parse_mode)
        .disable_link_preview(true)
        .await?;

    Ok(())
}
