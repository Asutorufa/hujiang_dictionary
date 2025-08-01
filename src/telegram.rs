use std::collections::HashSet;

use teloxide::{
    RequestError,
    dispatching::{DefaultKey, DpHandlerDescription},
    prelude::*,
    types::{ReplyParameters, Update, UserId},
    utils::command::BotCommands,
};

use crate::jp;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "jp -> cn")]
    JPCN(String),
    #[command(description = "cn -> jp")]
    CNJP(String),
    #[command(description = "get current user id")]
    UserID,
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
    pub maintainer: HashSet<UserId>,
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

    if !opt.maintainer.contains(&from_user) {
        return Ok(());
    }

    println!("new request from: {}", from_user);

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
    };

    bot.send_message(msg.chat.id, reply)
        .reply_parameters(ReplyParameters::default().allow_sending_without_reply())
        .await?;

    Ok(())
}
