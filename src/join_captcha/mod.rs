use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, User}, utils::html::escape,
};

use crate::HandlerResult;

mod math_captcha;
mod msgs;

#[derive(Serialize, Deserialize)]
struct CallBackData {
    data: String,
    btn_val: u8,
    user_id: UserId,
}

// Im putting so much data inside the callback because I want the bot to be database less and as portable as possible
pub async fn join_handler(bot: Bot, msg: Message, users: Vec<User>) -> HandlerResult {
    for user in users {
        let (question, answers) = math_captcha::generate_captcha();
        let text = format!(
            concat!(
                "سلام <a href=\"{}\">{}</a>\n",
                "به گروه {} خوش آمدید\n",
                "برای باز شدن گروه لطفا جواب درست رو انتخاب کنید\n",
                "<b>{}</b>"
            ),
            user.url(),
            escape(&user.full_name()),
            escape(msg.chat.title().unwrap_or("Unknown")),
            question,
        );

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
                    msgs::ADMIN_APPROVE,
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

pub async fn callback_handler(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let (Some(msg), Some(data)) = (q.message, q.data) {
        let callback_data: CallBackData = serde_json::from_str(&data)?;

        if callback_data.data == "admin_approve" {
            if !bot
                .get_chat_administrators(msg.chat.id)
                .await?
                .iter()
                .any(|c| c.user.id == q.from.id)
            {
                bot.answer_callback_query(q.id)
                    .text(msgs::ADMIN_ONLY)
                    .await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(msgs::ADMIN_APPROVED_USER)
                .await?;
        } else {
            if q.from.id != callback_data.user_id {
                bot.answer_callback_query(q.id)
                    .text(msgs::USER_DOESNT_MATCH)
                    .await?;
                return Ok(());
            }

            if !math_captcha::validate_captcha_answer(callback_data.data, callback_data.btn_val) {
                bot.answer_callback_query(q.id)
                    .text(msgs::WRONG_ANSWER)
                    .await?;
                bot.ban_chat_member(msg.chat.id, callback_data.user_id)
                    .await?;
                bot.delete_message(msg.chat.id, msg.id).await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(msgs::CORRECT_ANSWER)
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
