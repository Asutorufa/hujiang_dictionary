use crate::ai::Models;
use crate::d1::Queries;
use crate::{ai::Translator, opts::RunOpt};
use async_trait::async_trait;
use core::fmt;
use d1_orm::DatabaseExecutor;
use frankenstein::AsyncTelegramApi;
use frankenstein::client_reqwest::Bot;
use frankenstein::methods::{SendMessageParams, SetMyCommandsParams, SetWebhookParams};
use frankenstein::types::{ChatId, LinkPreviewOptions, MaybeInaccessibleMessage};
use hjdict::{en, google, jp, kotobanku, kr, weblio};
use log::*;
use std::sync::Arc;
use tg_bot_worker::{
    TelegramBot,
    utils::{html_escape, markdown_escape, split_message, vec_string_markdown_escape},
};

macro_rules! llm_parser {
    ($variant:ident) => {
        |_, arg: &str, quote: &str| {
            let full_text = match (quote.is_empty(), arg.is_empty()) {
                (false, false) => format!("{}\n{}", quote, arg),
                (false, true) => quote.to_string(),
                (true, false) => arg.to_string(),
                (true, true) => "".to_string(),
            };
            Ok((Self::$variant(full_text), None))
        }
    };
}

macro_rules! gg_parser {
    ($variant:ident) => {
        |_, arg: &str, quote: &str| {
            let mut parts = arg.splitn(2, ' ');
            let first = parts.next().unwrap_or("");
            let rest = parts.next().unwrap_or(quote).to_string();

            let args = first.split("_").collect::<Vec<_>>();

            let (src, target) = match args.len() > 1 {
                true => (Some(args[0].to_string()), args[1].to_string()),
                false => (None, first.to_string()),
            };

            if target.is_empty() {
                return Err("target language is empty".to_string());
            }

            if rest.is_empty() {
                return Err("text to translate is empty".to_string());
            }

            Ok((Self::$variant(src, target, rest), None))
        }
    };
}

tg_bot_worker::bot_commands! {
    #[derive(Debug, PartialEq, Eq)]
    pub enum Command {
        #[command(name = "jpcn", desc = "jp -> cn")]
        JPCN(String),
        #[command(name = "cnjp", desc = "cn -> jp")]
        CNJP(String),
        #[command(name = "kr", desc = "kr <-> cn")]
        KR(String),
        #[command(name = "en", desc = "en <-> cn")]
        EN(String),
        #[command(name = "weblio", desc = "weblio")]
        Weblio(String),
        #[command(name = "ktbk", desc = "コトバンク")]
        Ktbk(String),
        #[command(name = "gemma", desc = "gemma3 12b it", parser = llm_parser!(Gemma))]
        Gemma(String),
        #[command(name = "llama4", desc = "llama4 scout 17b 16e instruct", parser = llm_parser!(Llama4))]
        Llama4(String),
        #[command(name = "gpt", desc = "gpt-oss-20b", parser = llm_parser!(GPT))]
        GPT(String),
        #[command(name = "userid", desc = "get current user id", unit = true)]
        UserID,
        #[command(name = "gg", desc = "google translate, eg: /gg en_ja hello, /gg ja hello", parser = gg_parser!(GG))]
        GG(Option<String>, String, String),
        #[command(name = "gg1", desc = "google old translate api, eg: /gg1 en_ja hello, /gg1 ja hello", parser = gg_parser!(GG1))]
        GG1(Option<String>, String, String),
        #[command(name = "cfai", desc = "google translate, eg: /cfai en_ja hello, /cfai ja hello", parser = gg_parser!(CFAI))]
        CFAI(Option<String>, String, String),
        #[command(name = "random", desc = "get a random word from d1 database", unit = true)]
        Random,
        #[command(name = "save", desc = "save a message by word", parser = |_, arg: &str, quote: &str| Ok((Self::Save(arg.to_string(), quote.to_string()), None)))]
        Save(String, String),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        // Test empty command
        assert_eq!(
            Command::parse("", "", ""),
            Err("command is empty".to_string())
        );
        assert_eq!(
            Command::parse("/", "", ""),
            Err("command is empty".to_string())
        );
        assert_eq!(
            Command::parse("/@bot", "", ""),
            Err("command is empty".to_string())
        );

        // Test normal commands
        assert_eq!(
            Command::parse("/en", "hello", "").unwrap().0,
            Command::EN("hello".to_string())
        );
        assert_eq!(
            Command::parse("/jpcn", "hello", "").unwrap().0,
            Command::JPCN("hello".to_string())
        );
        assert_eq!(
            Command::parse("/cnjp", "hello", "").unwrap().0,
            Command::CNJP("hello".to_string())
        );
        assert_eq!(
            Command::parse("/kr", "hello", "").unwrap().0,
            Command::KR("hello".to_string())
        );
        assert_eq!(
            Command::parse("/weblio", "hello", "").unwrap().0,
            Command::Weblio("hello".to_string())
        );
        assert_eq!(
            Command::parse("/ktbk", "hello", "").unwrap().0,
            Command::Ktbk("hello".to_string())
        );
        assert_eq!(
            Command::parse("/random", "", "").unwrap().0,
            Command::Random
        );
        assert_eq!(
            Command::parse("/userid", "", "").unwrap().0,
            Command::UserID
        );

        // Test bot suffix
        assert_eq!(
            Command::parse("/en@my_bot", "hello", "").unwrap().0,
            Command::EN("hello".to_string())
        );

        // Test quote and argument
        assert_eq!(
            Command::parse("/en", "", "quote").unwrap().0,
            Command::EN("quote".to_string())
        );

        // Test GG command
        assert_eq!(
            Command::parse("/gg", "en_ja hello", "").unwrap().0,
            Command::GG(
                Some("en".to_string()),
                "ja".to_string(),
                "hello".to_string()
            )
        );

        assert_eq!(
            Command::parse("/gg", "ja hello", "quote").unwrap().0,
            Command::GG(None, "ja".to_string(), "hello".to_string())
        );

        // Test GG with quote
        assert_eq!(
            Command::parse("/gg", "ja", "quote").unwrap().0,
            Command::GG(None, "ja".to_string(), "quote".to_string())
        );

        // Test GG validation
        assert!(Command::parse("/gg", "", "").is_err());
        assert!(Command::parse("/gg", "ja", "").is_err());

        // Test LLM commands
        assert_eq!(
            Command::parse("/gemma", "arg", "quote").unwrap().0,
            Command::Gemma("quote\narg".to_string())
        );
        assert_eq!(
            Command::parse("/gemma", "arg", "").unwrap().0,
            Command::Gemma("arg".to_string())
        );
        assert_eq!(
            Command::parse("/gemma", "", "quote").unwrap().0,
            Command::Gemma("quote".to_string())
        );
        assert_eq!(
            Command::parse("/llama4", "arg", "quote").unwrap().0,
            Command::Llama4("quote\narg".to_string())
        );
        assert_eq!(
            Command::parse("/gpt", "arg", "quote").unwrap().0,
            Command::GPT("quote\narg".to_string())
        );

        // Test save command
        assert_eq!(
            Command::parse("/save", "arg", "quote").unwrap().0,
            Command::Save("arg".to_string(), "quote".to_string())
        );

        // Test unknown command
        assert_eq!(
            Command::parse("/unknown", "", ""),
            Err("not implemented".to_string())
        );
    }

    #[test]
    fn test_parse_callback_query_command() {
        assert_eq!(
            parse_callback_query_command("delete", "").unwrap().0,
            CallbackQueryCommand::Delete
        );
        assert_eq!(
            parse_callback_query_command("/save", "word").unwrap().0,
            CallbackQueryCommand::Save("word".to_string())
        );
        assert_eq!(
            parse_callback_query_command("remove", "word").unwrap().0,
            CallbackQueryCommand::Remove("word".to_string())
        );
        assert_eq!(
            parse_callback_query_command("unknown", ""),
            Err(Error("not implemented".to_string()))
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CallbackQueryCommand {
    Delete,
    Save(String),
    Remove(String),
}

pub struct BotHandler<T: DatabaseExecutor, T2: Translator> {
    pub opt: Arc<RunOpt<T, T2>>,
}

#[async_trait(?Send)]
impl<T: DatabaseExecutor, T2: Translator> TelegramBot for BotHandler<T, T2> {
    type Error = Error;

    async fn handle_command(
        &self,
        msg: Box<frankenstein::types::Message>,
        command: &str,
        argument: &str,
        quote: &str,
    ) -> Result<(), Self::Error> {
        let (cmd, text) = Command::parse(command, argument, quote).map_err(Error)?;

        info!("message command: {:?}, argument: {:?}", cmd, text);

        answer(self.opt.clone(), msg, cmd).await?;
        Ok(())
    }

    async fn handle_callback(
        &self,
        call_query: Box<frankenstein::types::CallbackQuery>,
        command: &str,
        argument: &str,
    ) -> Result<(), Self::Error> {
        let (cmd, text) = parse_callback_query_command(command, argument)?;

        info!("callback query command: {:?}, argument: {:?}", cmd, text);

        callback_query(self.opt.clone(), call_query, cmd).await?;
        Ok(())
    }
}

pub async fn handle<T: DatabaseExecutor, T2: Translator>(
    opt: Arc<RunOpt<T, T2>>,
    update: frankenstein::updates::Update,
) -> Result<(), Error> {
    let handler = BotHandler { opt };
    handler.handle_update(update).await
}

#[derive(Debug, PartialEq, Eq)]
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

pub async fn llm_answer<T: DatabaseExecutor, T2: Translator>(
    opt: Arc<RunOpt<T, T2>>,
    model: Models,
    v: String,
) -> (String, String) {
    if let Some(ai) = opt.workers_ai.as_ref() {
        let req = crate::ai::TranslateRequest {
            model: model.as_str(),
            query: &v,
            chars_limit: true,
            ..Default::default()
        };
        match crate::ai::translate(ai, req).await {
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(x) => (v, markdown_escape(x.content.as_str()).to_string()),
        }
    } else {
        (v, "workers_ai not available".to_string())
    }
}

pub async fn answer<T: DatabaseExecutor, T2: Translator>(
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
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(v) => (
                word,
                vec_string_markdown_escape(
                    &v.iter().map(|x| x.markdown()).collect::<Vec<String>>(),
                ),
            ),
        },
        Command::EN(word) => match en::get(word.as_str()).await {
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(v) => (
                word,
                vec_string_markdown_escape(
                    &v.iter().map(|x| x.markdown()).collect::<Vec<String>>(),
                ),
            ),
        },
        Command::JPCN(word) => match jp::get(word.as_str(), "jc").await {
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(v) => (
                word,
                vec_string_markdown_escape(
                    &v.iter().map(|x| x.markdown()).collect::<Vec<String>>(),
                ),
            ),
        },
        Command::KR(word) => match kr::get(word.as_str()).await {
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(v) => (
                word,
                vec_string_markdown_escape(
                    &v.iter().map(|x| x.markdown()).collect::<Vec<String>>(),
                ),
            ),
        },
        Command::Ktbk(word) => match kotobanku::get(word.as_str()).await {
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(v) => {
                let reply = vec_string_markdown_escape(&v);
                if reply.len() > 4096 {
                    (word, markdown_escape(&v[0]).to_string())
                } else {
                    (word, reply)
                }
            }
        },
        Command::Weblio(word) => match weblio::get(word.as_str()).await {
            Err(e) => (
                "".to_string(),
                markdown_escape(e.to_string().as_str()).to_string(),
            ),
            Ok(v) => {
                let reply = vec_string_markdown_escape(&v);
                if reply.len() > 4096 {
                    (word, markdown_escape(&v[0]).to_string())
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
            match google::translatev2(text.as_ref(), from, to.as_ref()).await {
                Err(e) => (
                    "".to_string(),
                    markdown_escape(e.to_string().as_str()).to_string(),
                ),
                Ok(v) => (
                    text,
                    markdown_escape(google::merge_translation(v).as_str()).to_string(),
                ),
            }
        }
        Command::GG1(from, to, text) => {
            match google::translate(text.as_ref(), from, to.as_ref()).await {
                Err(e) => (
                    "".to_string(),
                    markdown_escape(e.to_string().as_str()).to_string(),
                ),
                Ok(v) => (
                    text,
                    markdown_escape(google::merge_translation(v).as_str()).to_string(),
                ),
            }
        }
        Command::CFAI(from, to, text) => {
            match opt.translator.m2m100_1_2b(text.as_ref(), from, to).await {
                Err(e) => (
                    "".to_string(),
                    markdown_escape(e.to_string().as_str()).to_string(),
                ),
                Ok(v) => (text, markdown_escape(v.as_str()).to_string()),
            }
        }
        Command::Gemma(v) => llm_answer(opt.clone(), Models::Gemma3_12bIt, v).await,
        Command::Llama4(v) => llm_answer(opt.clone(), Models::Llama4Scout17B16EInstruct, v).await,
        Command::GPT(v) => llm_answer(opt.clone(), Models::GPTOss20B, v).await,
        Command::Save(word, explain) => {
            if word.is_empty() || explain.is_empty() {
                ("".to_string(), "empty word or explain".to_string())
            } else {
                parse_mode = frankenstein::ParseMode::Html;
                match opt
                    .d1
                    .execute(Queries::SaveWord {
                        word: &word,
                        explain: &explain,
                        word_type: 0,
                        example: "",
                    })
                    .await
                {
                    Err(e) => ("".to_string(), e.to_string()),
                    Ok(_) => (
                        "".to_string(),
                        format!("save <b>{}</b> to d1 database", html_escape(word.as_str())),
                    ),
                }
            }
        }
        Command::Random => match crate::d1::random_word(&opt.d1).await.as_ref() {
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

    let (word, reply) = if reply.is_empty() {
        ("", "can't found explain")
    } else {
        (word.as_ref(), reply.as_ref())
    };

    for v in split_message(reply, 4096) {
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
                                if word.len() < 58 { word } else { "" }
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

pub async fn callback_query<T: DatabaseExecutor, T2: Translator>(
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

            if let Err(e) = opt
                .d1
                .execute(Queries::SaveWord {
                    word: &v,
                    explain: &explain,
                    word_type: 0,
                    example: "",
                })
                .await
            {
                error!("save word failed: {}", e);
                return Ok(());
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
            if let Err(e) = opt.d1.execute(Queries::DeleteWord { word: &v }).await {
                error!("delete word failed: {}", e);
                return Ok(());
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

pub async fn send_random_word<T: DatabaseExecutor, T2: Translator>(
    opt: Arc<RunOpt<T, T2>>,
) -> Result<(), frankenstein::Error> {
    let reply = match crate::d1::random_word(&opt.d1).await {
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
                .chat_id(frankenstein::types::ChatId::Integer(opt.matainer))
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
            .commands(Command::bot_commands())
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
