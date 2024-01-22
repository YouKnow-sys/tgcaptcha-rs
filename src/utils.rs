use anyhow::Result;
use teloxide::{
    requests::Requester,
    types::{ChatId, UserId},
    Bot,
};

use crate::config::GroupSettings;

/// check whatever if user is one of the custom admins or chat admins
pub async fn is_user_admin(
    bot: &Bot,
    chat_id: ChatId,
    user_id: UserId,
    group_settings: &GroupSettings,
) -> Result<bool> {
    Ok(match group_settings.custom_admins {
        Some(ref list) => list.contains(&user_id),
        None => bot
            .get_chat_administrators(chat_id)
            .await?
            .iter()
            .any(|c| c.user.id == user_id),
    })
}
