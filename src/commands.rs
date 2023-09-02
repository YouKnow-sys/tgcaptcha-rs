use std::sync::Arc;

use teloxide::{
    requests::Requester,
    types::{Me, Message},
    utils::command::BotCommands,
    Bot,
};

use crate::{config::GroupsConfig, HandlerResult};

#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Check the bot status")]
    Status,
}

pub async fn command_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    msg: Message,
    me: Me,
    text: String,
) -> HandlerResult {
    if let Some(user) = msg.from() {
        if config.is_group_allowed(&msg.chat.id) {
            let is_allowed = match &config.get(&msg.chat.id).custom_admins {
                Some(list) => list.contains(&user.id),
                None => bot
                    .get_chat_administrators(msg.chat.id)
                    .await?
                    .iter()
                    .any(|c| c.user.id == user.id),
            };

            if is_allowed {
                match BotCommands::parse(text.as_str(), me.username()) {
                    Ok(Command::Help) => {
                        bot.send_message(msg.chat.id, Command::descriptions().to_string())
                            .await?;
                    }
                    Ok(Command::Status) => {
                        bot.send_message(msg.chat.id, "Im Up and running!").await?;
                    }

                    Err(_) => (),
                };
            }
        }
    }

    Ok(())
}
