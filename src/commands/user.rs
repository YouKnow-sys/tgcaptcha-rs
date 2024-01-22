use std::sync::Arc;

use anyhow::Result;

use teloxide::{types::Message, utils::command::BotCommands, Bot};

use crate::{config::GroupsConfig, crates::CratesAction, utils};

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "Available user commands:")]
pub enum UserCommands {
    #[command(description = "Search for a crate in crates.io")]
    Crate(String),
    #[command(description = "Search for a crate doc")]
    Doc(String),
}

pub async fn user_command_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    msg: Message,
    client: reqwest::Client,
    cmd: UserCommands,
) -> Result<()> {
    if let Some(user) = msg.from() {
        if config.is_group_allowed(msg.chat.id) {
            let group_settings = config.get(msg.chat.id);
            if !(group_settings.crates_commands
                || utils::is_user_admin(&bot, msg.chat.id, user.id, config.get(msg.chat.id))
                    .await?)
            {
                return Ok(());
            }

            match cmd {
                UserCommands::Crate(crate_name) => {
                    crate::crates::crates_command_handler(
                        bot,
                        msg,
                        client,
                        group_settings,
                        &crate_name,
                        CratesAction::CrateLookup,
                    )
                    .await?
                }
                UserCommands::Doc(query) => {
                    crate::crates::crates_command_handler(
                        bot,
                        msg,
                        client,
                        group_settings,
                        &query,
                        CratesAction::DocLookup,
                    )
                    .await?
                }
            };
        }
    }

    Ok(())
}
