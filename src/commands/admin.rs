use std::{sync::Arc, time::Instant};

use anyhow::Result;

use teloxide::{
    payloads::SendMessageSetters, requests::Requester, types::Message, utils::command::BotCommands,
    Bot,
};

use crate::{config::GroupsConfig, utils};

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "Available admin commands:")]
pub enum AdminCommands {
    #[command(description = "Check the bot status")]
    Status,
    #[command(description = "Show the uptime of the bot")]
    Uptime,
    #[command(description = "Bot source code")]
    SourceCode,
}

pub async fn admin_command_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    msg: Message,
    instant: Instant,
    cmd: AdminCommands,
) -> Result<()> {
    if let Some(user) = msg.from() {
        if config.is_group_allowed(msg.chat.id)
            && utils::is_user_admin(&bot, msg.chat.id, user.id, config.get(msg.chat.id)).await?
        {
            match cmd {
                AdminCommands::Status => {
                    bot.send_message(msg.chat.id, "I'm Up and running!")
                        .reply_to_message_id(msg.id)
                        .await?;
                }
                AdminCommands::Uptime => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Bot uptime: {}",
                            humantime::format_duration(instant.elapsed())
                        ),
                    )
                    .reply_to_message_id(msg.id)
                    .await?;
                }
                AdminCommands::SourceCode => {
                    bot.send_message(
                        msg.chat.id,
                        concat!(
                            "You can find tgcaptcha-rs source code here\n",
                            env!("CARGO_PKG_REPOSITORY")
                        ),
                    )
                    .reply_to_message_id(msg.id)
                    .await?;
                }
            };
        }
    }

    Ok(())
}
