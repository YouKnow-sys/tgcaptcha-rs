use teloxide::{
    requests::Requester,
    types::{Me, Message},
    utils::command::BotCommands,
    Bot,
};

use crate::HandlerResult;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Check the bot status")]
    Status,
}

pub async fn command_handler(bot: Bot, msg: Message, me: Me, text: String) -> HandlerResult {
    if let Some(user) = msg.from() {
        if bot
            .get_chat_administrators(msg.chat.id)
            .await?
            .iter()
            .any(|c| c.user.id == user.id)
        {
            match BotCommands::parse(text.as_str(), me.username()) {
                Ok(Command::Help) => {
                    bot.send_message(msg.chat.id, Command::descriptions().to_string())
                        .await?;
                }
                Ok(Command::Status) => {
                    bot.send_message(msg.chat.id, "Im Up and running!").await?;
                }

                Err(_) => (),
            }
        }
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
