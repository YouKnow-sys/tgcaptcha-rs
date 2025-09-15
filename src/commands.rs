use std::{sync::Arc, time::Instant};

use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Me, Message, ReplyParameters},
    utils::command::BotCommands,
    Bot,
};

use crate::{config::GroupsConfig, helpers, HandlerResult};

#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Check the bot status")]
    Status,
    #[command(description = "Show the uptime of the bot")]
    Uptime,
    #[command(description = "Bot source code")]
    SourceCode,
}

pub async fn command_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    msg: Message,
    me: Me,
    text: String,
    instant: Instant,
) -> HandlerResult {
    let Some(user) = msg.from else {
        return Ok(());
    };

    if !config.is_group_allowed(msg.chat.id) {
        return Ok(());
    }

    if !helpers::is_allowed_admin(user.id, &bot, config.get(msg.chat.id), msg.chat.id).await? {
        return Ok(());
    }

    let Ok(command) = BotCommands::parse(text.as_str(), me.username()) else {
        return Ok(());
    };

    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .reply_parameters(ReplyParameters::new(msg.id).allow_sending_without_reply())
                .await?;
        }
        Command::Status => {
            bot.send_message(msg.chat.id, "I'm up and running!")
                .reply_parameters(ReplyParameters::new(msg.id).allow_sending_without_reply())
                .await?;
        }
        Command::Uptime => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "Bot uptime: {}",
                    humantime::format_duration(instant.elapsed())
                ),
            )
            .reply_parameters(ReplyParameters::new(msg.id).allow_sending_without_reply())
            .await?;
        }
        Command::SourceCode => {
            bot.send_message(
                msg.chat.id,
                concat!(
                    "You can find tgcaptcha-rs source code here\n",
                    env!("CARGO_PKG_REPOSITORY")
                ),
            )
            .reply_parameters(ReplyParameters::new(msg.id).allow_sending_without_reply())
            .await?;
        }
    }

    Ok(())
}
