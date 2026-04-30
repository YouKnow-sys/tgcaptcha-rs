use std::sync::Arc;

use anyhow::Context;
use chrono::Utc;
use sqlx::SqlitePool;
use storage::ChatStorage;
use teloxide::{
    prelude::*,
    types::{
        ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage,
        ParseMode, ReplyParameters, User,
    },
};

use crate::{config::GroupsConfig, helpers, HandlerResult};
pub use captcha::*;
use storage::JoinCheckData;

mod captcha;
pub mod storage;

const ANSWERS_COUNT: usize = 4;
const ADMIN_APPROVE_CB: &str = "admin_approve";
const DELETE_KEYBOARD_CB: &str = "delete_keyboard";

pub async fn join_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    msg: Message,
    users: Vec<User>,
    pool: SqlitePool,
) -> HandlerResult {
    if !config.is_group_allowed(msg.chat.id) {
        log::warn!(
            "unknown chat {} with id {}",
            msg.chat.title().unwrap_or("N/A"),
            msg.chat.id
        );

        if let Err(e) = bot
            .send_message(
                msg.chat.id,
                &config.get(msg.chat.id).messages.unauthorized_group,
            )
            .await
        {
            log::error!(
                "failed to send message to unauthorized chat {}: {}",
                msg.chat.id,
                e
            );
        }

        bot.leave_chat(msg.chat.id)
            .await
            .context("failed to leave unauthorized chat")?;

        return Ok(());
    }

    let chat_cfg = config.get(msg.chat.id);
    let storage = ChatStorage::new(pool, msg.chat.id);

    if chat_cfg.remove_join_messages {
        if let Err(e) = bot.delete_message(msg.chat.id, msg.id).await {
            log::warn!("failed to delete join service message: {e}");
        }
    }

    for user in users {
        if user.is_bot {
            continue;
        }

        let (question, answers) = MathQuestion::generate_question::<ANSWERS_COUNT>();

        let welcome_msg = chat_cfg.messages.create_welcome_msg(
            &user,
            &helpers::get_safe_chat_name(chat_cfg.custom_chat_name.as_deref(), &msg.chat),
            question,
        );

        bot.restrict_chat_member(msg.chat.id, user.id, ChatPermissions::empty())
            .await
            .context("failed to restrcit chat member")?;

        let message_id = bot
            .send_message(msg.chat.id, welcome_msg)
            .parse_mode(ParseMode::Html)
            .reply_parameters(ReplyParameters::new(msg.id).allow_sending_without_reply())
            .reply_markup(create_answer_buttons(
                answers,
                &chat_cfg.messages.admin_approve,
            ))
            .await
            .context("failed to send captcha question to user")?
            .id;

        storage
            .add(JoinCheckData {
                user_id: user.id,
                chat_id: msg.chat.id,
                message_id,
                question,
                is_passed: false,
                expires_at: Utc::now() + chat_cfg.ban_after,
            })
            .await
            .context("failed to add join check data to database")?;

        tokio::spawn({
            let bot = bot.clone();
            let ban_after = chat_cfg.ban_after;
            let config = config.clone();
            let storage = storage.clone();

            async move {
                tokio::time::sleep(ban_after).await;
                match storage.get_and_delete(message_id).await {
                    Ok(Some(data)) => {
                        if !data.is_passed {
                            if let Err(e) = bot.ban_chat_member(msg.chat.id, data.user_id).await {
                                log::error!("failed to ban the member after timeout: {e}");

                                let chat_cfg = config.get(msg.chat.id);
                                match bot
                                    .edit_message_text(
                                        data.chat_id,
                                        data.message_id,
                                        chat_cfg.messages.create_removeing_user_failed(&user),
                                    )
                                    .reply_markup(create_delete_message_keyboard(
                                        &chat_cfg.messages.delete_message,
                                    ))
                                    .await
                                {
                                    Ok(_) => return,
                                    Err(e) => log::error!("failed to edit message: {e}"),
                                }
                            }

                            if let Err(e) = bot.delete_message(data.chat_id, data.message_id).await
                            {
                                log::error!("failed to delete message after timeout: {e}");
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => log::error!("failed to get join check data from database: {e}"),
                }
            }
        });
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    config: Arc<GroupsConfig>,
    cbq: CallbackQuery,
    pool: SqlitePool,
) -> HandlerResult {
    if let (Some(MaybeInaccessibleMessage::Regular(msg)), Some(data)) = (cbq.message, cbq.data) {
        if !config.is_group_allowed(msg.chat.id) {
            return Ok(());
        }

        let Some(permissions) = bot.get_chat(msg.chat.id).await?.permissions() else {
            anyhow::bail!("failed to get the group permissions")
        };

        let chat_cfg = config.get(msg.chat.id);
        let storage = ChatStorage::new(pool, msg.chat.id);

        let Some(join_data) = storage
            .get(msg.id)
            .await
            .context("failed to get join check data from database")?
        else {
            anyhow::bail!("can't find the join check data in database");
        };

        match data.as_str() {
            ADMIN_APPROVE_CB => {
                if !helpers::is_allowed_admin(cbq.from.id, &bot, chat_cfg, msg.chat.id).await? {
                    bot.answer_callback_query(cbq.id)
                        .text(&chat_cfg.messages.admin_only_error)
                        .await?;

                    return Ok(());
                }

                bot.answer_callback_query(cbq.id)
                    .text(&chat_cfg.messages.admin_approved_user)
                    .await?;
            }
            DELETE_KEYBOARD_CB => {
                if !helpers::is_allowed_admin(cbq.from.id, &bot, chat_cfg, msg.chat.id).await? {
                    bot.answer_callback_query(cbq.id)
                        .text(&chat_cfg.messages.admin_only_error)
                        .await?;

                    return Ok(());
                }

                bot.delete_message(msg.chat.id, msg.id).await?;

                return Ok(());
            }
            data => {
                if cbq.from.id != join_data.user_id {
                    bot.answer_callback_query(cbq.id)
                        .text(&chat_cfg.messages.user_doesnt_match_error)
                        .await?;

                    return Ok(());
                }

                if !join_data.question.validate_answer(data.parse()?) {
                    bot.answer_callback_query(cbq.id)
                        .text(&chat_cfg.messages.wrong_answer)
                        .await?;

                    bot.ban_chat_member(msg.chat.id, join_data.user_id).await?;
                    bot.delete_message(msg.chat.id, msg.id).await?;

                    return Ok(());
                }

                bot.answer_callback_query(cbq.id)
                    .text(&chat_cfg.messages.correct_answer)
                    .await?;
            }
        }

        storage
            .mark_as_passed(join_data.message_id)
            .await
            .context("failed to mark join data as passed")?;

        bot.restrict_chat_member(msg.chat.id, join_data.user_id, permissions)
            .await?;

        bot.delete_message(msg.chat.id, msg.id).await?;
    }

    Ok(())
}

fn create_answer_buttons(
    answers: [u8; ANSWERS_COUNT],
    admin_approve: &str,
) -> InlineKeyboardMarkup {
    let answers = answers
        .into_iter()
        .map(|a| {
            let a = a.to_string();
            InlineKeyboardButton::callback(a.clone(), a)
        })
        .collect();

    InlineKeyboardMarkup::new([
        answers,
        vec![InlineKeyboardButton::callback(
            admin_approve,
            ADMIN_APPROVE_CB,
        )],
    ])
}

fn create_delete_message_keyboard(delete_message: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([vec![InlineKeyboardButton::callback(
        delete_message,
        DELETE_KEYBOARD_CB,
    )]])
}
