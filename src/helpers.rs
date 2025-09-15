use teloxide::{
    prelude::Requester,
    types::{Chat, ChatId, UserId},
    utils::html,
    Bot,
};

use crate::config::GroupSettings;

pub fn get_safe_chat_name(custom_chat_name: Option<&str>, chat: &Chat) -> String {
    let name = custom_chat_name.unwrap_or(chat.title().unwrap_or("N/A"));

    if !name.contains(['&', '<', '>']) {
        return name.to_owned();
    }

    html::escape(name)
}

pub async fn is_allowed_admin(
    user_id: UserId,
    bot: &Bot,
    chat_cfg: &GroupSettings,
    chat_id: ChatId,
) -> Result<bool, teloxide::RequestError> {
    let allowed = match &chat_cfg.custom_admins {
        Some(list) => list.contains(&user_id),
        None => bot
            .get_chat_administrators(chat_id)
            .await?
            .iter()
            .any(|c| c.user.id == user_id),
    };

    Ok(allowed)
}
