use std::sync::Arc;

use anyhow::Result;
use teloxide::{
    payloads::SendMessageSetters, requests::Requester, types::Message, utils::command::BotCommands,
    Bot,
};

pub use admin::*;
pub use user::*;

use crate::{config::GroupsConfig, utils};

mod admin;
mod user;

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Commands {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Admin commands help", rename = "admin_help")]
    AdminHelp,
    #[command(description = "Users commands help", rename = "user_help")]
    UserHelp,
}

pub async fn command_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    msg: Message,
    cmd: Commands,
) -> Result<()> {
    if let Some(user) = msg.from() {
        if config.is_group_allowed(msg.chat.id) {
            match cmd {
                Commands::Help => {
                    let mut help = Commands::descriptions().to_string();
                    if !config.get(msg.chat.id).crates_commands {
                        help.push_str(
                            "\n\nNote: Admins of this group disabled User commands for users.",
                        );
                    }

                    bot.send_message(msg.chat.id, help)
                        .reply_to_message_id(msg.id)
                        .await?;
                }
                Commands::AdminHelp => {
                    if utils::is_user_admin(&bot, msg.chat.id, user.id, config.get(msg.chat.id))
                        .await?
                    {
                        bot.send_message(msg.chat.id, AdminCommands::descriptions().to_string())
                            .reply_to_message_id(msg.id)
                            .await?;
                    }
                }
                Commands::UserHelp => {
                    if config.get(msg.chat.id).crates_commands
                        || utils::is_user_admin(&bot, msg.chat.id, user.id, config.get(msg.chat.id))
                            .await?
                    {
                        bot.send_message(msg.chat.id, UserCommands::descriptions().to_string())
                            .reply_to_message_id(msg.id)
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
