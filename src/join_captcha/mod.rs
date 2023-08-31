use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, User},
    utils::html::escape,
};

use crate::{config::AppConfig, HandlerResult};

mod math_captcha;

#[derive(Serialize, Deserialize)]
struct CallBackData {
    data: String,
    btn_val: u8,
    user_id: UserId,
}

pub async fn join_handler(
    bot: Bot,
    app_config: Arc<AppConfig>,
    msg: Message,
    users: Vec<User>,
) -> HandlerResult {
    let Some(chat_config) = app_config.allowed_chats.iter().find(|c| c.id == msg.chat.id) else {
        log::error!("Chat not found: {:?}", msg.chat);
        return Ok(());
    };

    for user in users {
        let (question, answers) = math_captcha::generate_captcha();

        let welcome_msg = chat_config
            .messages
            .new_user_template
            .clone()
            .replace(
                "{TAGUSER}",
                &format!(
                    "<a href=\"{}\">{}</a>\n",
                    user.url(),
                    escape(&user.full_name())
                ),
            )
            .replace(
                "{CHATNAME}",
                &escape(if let Some(ref title) = chat_config.override_chat_name {
                    title
                } else {
                    msg.chat.title().unwrap_or_default()
                }),
            );

        let text = format!("{}\n<b>{}</b>", welcome_msg, question);

        bot.restrict_chat_member(msg.chat.id, user.id, !ChatPermissions::SEND_MESSAGES)
            .await?;
        bot.send_message(msg.chat.id, text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_to_message_id(msg.id)
            .reply_markup(InlineKeyboardMarkup::new([
                answers
                    .into_iter()
                    .map(|a| {
                        InlineKeyboardButton::callback(
                            a.to_string(),
                            serde_json::to_string(&CallBackData {
                                data: question.to_owned(),
                                btn_val: a,
                                user_id: user.id,
                            })
                            .expect("Failed to serialize callback data"),
                        )
                    })
                    .collect(),
                vec![InlineKeyboardButton::callback(
                    &chat_config.messages.admin_approve,
                    serde_json::to_string(&CallBackData {
                        data: "admin_approve".to_owned(),
                        btn_val: 0,
                        user_id: user.id,
                    })
                    .expect("Failed to serialize callback data"),
                )],
            ]))
            .await?;
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    app_config: Arc<AppConfig>,
    q: CallbackQuery,
) -> HandlerResult {
    if let (Some(msg), Some(data)) = (q.message, q.data) {
        let Some(chat_config) = app_config.allowed_chats.iter().find(|c| c.id == msg.chat.id) else {
            return Ok(());
        };

        let callback_data: CallBackData = serde_json::from_str(&data)?;

        if callback_data.data == "admin_approve" {
            if !bot
                .get_chat_administrators(msg.chat.id)
                .await?
                .iter()
                .any(|c| c.user.id == q.from.id)
            {
                bot.answer_callback_query(q.id)
                    .text(&chat_config.messages.admin_only_error)
                    .await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(&chat_config.messages.admin_approved_user)
                .await?;
        } else {
            if q.from.id != callback_data.user_id {
                bot.answer_callback_query(q.id)
                    .text(&chat_config.messages.user_doesnt_match_error)
                    .await?;
                return Ok(());
            }

            if !math_captcha::validate_captcha_answer(callback_data.data, callback_data.btn_val) {
                bot.answer_callback_query(q.id)
                    .text(&chat_config.messages.wrong_answer)
                    .await?;
                bot.ban_chat_member(msg.chat.id, callback_data.user_id)
                    .await?;
                bot.delete_message(msg.chat.id, msg.id).await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(&chat_config.messages.correct_answer)
                .await?;
        }

        bot.restrict_chat_member(
            msg.chat.id,
            callback_data.user_id,
            ChatPermissions::SEND_MESSAGES,
        )
        .await?;
        bot.delete_message(msg.chat.id, msg.id).await?;
    }

    Ok(())
}
