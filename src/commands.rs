use std::sync::Arc;

use teloxide::{
    requests::Requester,
    types::{Me, Message},
    utils::command::BotCommands,
    Bot,
};

use crate::{
    config::{AppConfig, ChatAdmins},
    HandlerResult,
};

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
    app_config: Arc<AppConfig>,
    msg: Message,
    me: Me,
    text: String,
) -> HandlerResult {
    let mut is_allowed = false;
    if let Some(user) = msg.from() {
        let admins = app_config.allowed_chats.iter().map(|c| &c.admins);

        for admin in admins {
            let result = match admin {
                ChatAdmins::Explicit(list) => list.contains(&user.id),
                ChatAdmins::AllAdmins => {
                    let chat_admins = bot.get_chat_administrators(msg.chat.id).await?;
                    chat_admins.iter().any(|i| i.user.id == user.id)
                }
            };

            if result {
                is_allowed = true;
                break;
            }
        }
    }

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
    Ok(())
}

// Maybe todo, if wanted the bot to be more customizable
// fn make_settings_keyboard() -> InlineKeyboardMarkup {
//     InlineKeyboardMarkup::new(
//         vec![
//             vec![InlineKeyboardButton::callback("Update Welcome msg", "update_welcome")],
//             vec![InlineKeyboardButton::callback("Update success msg", "update_success")],
//             vec![InlineKeyboardButton::callback("Update fail msg", "update_fail")],
//             vec![
//                 InlineKeyboardButton::callback("➖", "welcome_timeout_minus"),
//                 InlineKeyboardButton::callback("Welcome timout", "welcome_timeout"),
//                 InlineKeyboardButton::callback("➕", "welcome_timeout_plus"),
//             ],
//             vec![
//                 InlineKeyboardButton::callback("➖", "ban_durations_minus"),
//                 InlineKeyboardButton::callback("Ban durations", "ban_durations"),
//                 InlineKeyboardButton::callback("➕", "ban_durations_plus"),
//             ],
//         ]
//     )
// }
