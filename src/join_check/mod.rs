use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, User},
    utils::html::escape,
};

use crate::{config::BotConfig, DialogueData, GroupDialogue, HandlerError, HandlerResult};
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
    config: Arc<BotConfig>,
    dialogue: GroupDialogue,
    msg: Message,
    users: Vec<User>,
) -> HandlerResult {
    if !config.is_group_allowed(&msg.chat.id) {
        log::error!("Chat not found: {:?}", msg.chat);

        bot.send_message(msg.chat.id, &config.get(&msg.chat.id).messages.unauthorized_group).await?;
        bot.leave_chat(msg.chat.id).await?;

        return Ok(());
    }

    let chat_cfg = config.get(&msg.chat.id);

    for user in users {
        if user.is_bot {
            continue;
        }

        let (question, answers) = Question::generate_question();

        let welcome_msg = chat_cfg.messages.create_welcome_msg(
            &user,
            &escape(if let Some(ref title) = chat_cfg.custom_chat_name {
                title
            } else {
                msg.chat.title().unwrap_or_default()
            }),
            question.clone(),
        );

        bot.restrict_chat_member(msg.chat.id, user.id, !ChatPermissions::SEND_MESSAGES)
            .await?;

        let answers_btn = answers
            .into_iter()
            .map(|a| {
                let a = a.to_string();
                InlineKeyboardButton::callback(a.clone(), a)
            })
            .collect();

        let msg_id = bot
            .send_message(msg.chat.id, welcome_msg)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_to_message_id(msg.id)
            .reply_markup(InlineKeyboardMarkup::new([
                answers_btn,
                vec![InlineKeyboardButton::callback(
                    &chat_cfg.messages.admin_approve,
                    "admin_approve",
                )],
            ]))
            .await?
            .id;

        let dialogue = dialogue
            .get()
            .await?
            .ok_or::<HandlerError>("Can't find group dialogue in memory".into())?;
        dialogue.insert(msg_id, DialogueData::new(user.id, question));

        tokio::spawn({
            let bot = bot.clone();
            let ban_after = chat_cfg.ban_after;
            async move {
                tokio::time::sleep(ban_after).await;
                if let Some((_, data)) = dialogue.remove(&msg_id) {
                    if !data.passed {
                        bot.ban_chat_member(msg.chat.id, data.user_id)
                            .await
                            .expect("Failed to ban memeber after timeout");
                        bot.delete_message(msg.chat.id, msg_id)
                            .await
                            .expect("Failed to delete msg after timeout");
                    }
                }
            }
        });
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    config: Arc<BotConfig>,
    dialogue: GroupDialogue,
    q: CallbackQuery,
) -> HandlerResult {
    if let (Some(msg), Some(data)) = (q.message, q.data) {
        if !config.is_group_allowed(&msg.chat.id) {
            return Ok(());
        }
        
        let chat_cfg = config.get(&msg.chat.id);

        let dlg_map = dialogue
            .get()
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
                    .text(&chat_cfg.messages.admin_only_error)
                    .await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(&chat_cfg.messages.admin_approved_user)
                .await?;
        } else {
            if q.from.id != dlg_data.user_id {
                bot.answer_callback_query(q.id)
                    .text(&chat_cfg.messages.user_doesnt_match_error)
                    .await?;

                return Ok(());
            }

            if !dlg_data.question.validate_question(data.parse()?) {
                bot.answer_callback_query(q.id)
                    .text(&chat_cfg.messages.wrong_answer)
                    .await?;

                bot.ban_chat_member(msg.chat.id, dlg_data.user_id).await?;
                bot.delete_message(msg.chat.id, msg.id).await?;

                return Ok(());
            }

            bot.answer_callback_query(q.id)
                .text(&chat_cfg.messages.correct_answer)
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
