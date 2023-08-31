use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, User},
    utils::html::escape,
};

use crate::{config::AppConfig, HandlerResult, GroupDialogue, HandlerError, DialogueData};
pub use math_captcha::Question;

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
    dialogue: GroupDialogue,
    msg: Message,
    users: Vec<User>,
) -> HandlerResult {
    let Some(chat_config) = app_config.allowed_chats.iter().find(|c| c.id == msg.chat.id) else {
        log::error!("Chat not found: {:?}", msg.chat);

        bot.send_message(msg.chat.id, "This group isn't authorized. Goodbye!").await?;
        bot.leave_chat(msg.chat.id).await?;

        return Ok(());
    };

    for user in users {
        if user.is_bot { continue }

        let (question, answers) = Question::generate_question();

        let welcome_msg = chat_config
            .messages
            .new_user_template
            .clone()
            .replace(
                "{TAGUSER}",
                &format!(
                    "<a href=\"{}\">{}</a>",
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
        let msg_id = bot.send_message(msg.chat.id, text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_to_message_id(msg.id)
            .reply_markup(InlineKeyboardMarkup::new([
                answers
                    .into_iter()
                    .map(|a| {
                        let a = a.to_string();
                        InlineKeyboardButton::callback(a.clone(), a)
                    })
                    .collect(),
                vec![InlineKeyboardButton::callback(&chat_config.messages.admin_approve, "admin_approve")],
            ]))
            .await?
            .id;

        let dialogue = dialogue.get()
            .await?
            .ok_or::<HandlerError>("Can't find group dialogue in memory".into())?;
        dialogue.insert(msg_id, DialogueData::new(user.id, question));

        tokio::spawn({
            let bot = bot.clone();
            let ban_after = chat_config.ban_after;
            async move {
                tokio::time::sleep(Duration::from_secs(ban_after)).await;
                if let Some((_, data)) = dialogue.remove(&msg_id) {
                    if !data.passed {
                        bot.ban_chat_member(msg.chat.id, data.user_id).await.expect("Failed to ban memeber after timeout");
                        bot.delete_message(msg.chat.id, msg_id).await.expect("Failed to delete msg after timeout");
                    }
                }
            }
        });
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    app_config: Arc<AppConfig>,
    dialogue: GroupDialogue,
    q: CallbackQuery,
) -> HandlerResult {
    if let (Some(msg), Some(data)) = (q.message, q.data) {
        let Some(chat_config) = app_config.allowed_chats.iter().find(|c| c.id == msg.chat.id) else {
            return Ok(());
        };

        let dlg_map = dialogue.get()
            .await?
            .ok_or::<HandlerError>("Can't find group dialogue in memory".into())?;
        let mut dlg_data = dlg_map
            .get_mut(&msg.id)
            .ok_or::<HandlerError>("Can't find message id in group dialogue".into())?;

        if data == "admin_approve" {
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
            if q.from.id != dlg_data.user_id {
                bot.answer_callback_query(q.id)
                    .text(&chat_config.messages.user_doesnt_match_error)
                    .await?;
                return Ok(());
            }

            if !dlg_data.question.validate_question(data.parse()?) {
                bot.answer_callback_query(q.id)
                    .text(&chat_config.messages.wrong_answer)
                    .await?;
                bot.ban_chat_member(msg.chat.id, dlg_data.user_id)
                    .await?;
                bot.delete_message(msg.chat.id, msg.id).await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(&chat_config.messages.correct_answer)
                .await?;
        }

        dlg_data.passed = true;

        bot.restrict_chat_member(
            msg.chat.id,
            dlg_data.user_id,
            ChatPermissions::SEND_MESSAGES,
        )
        .await?;
        bot.delete_message(msg.chat.id, msg.id).await?;
    }

    Ok(())
}
