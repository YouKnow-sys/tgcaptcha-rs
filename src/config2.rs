use std::{collections::HashMap, time::Duration};

use config::{Config as PConfing, ConfigError, File, Environment};
use serde::{Serialize, Deserialize};
use teloxide::{types::{ChatId, UserId, User}, utils::html::user_mention_or_link};

use crate::join_check::Question;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// List of allowed group, if `None` bot will allow all groups
    pub allowed_groups: Option<Vec<ChatId>>,
    fallback_group_settings: GroupSettings,
    groups_settings: HashMap<ChatId, GroupSettings>,
}

impl Config {
    pub fn try_read() -> Result<Config, ConfigError> {
        PConfing::builder()
            .add_source(File::with_name("Config.toml").required(false))
            .add_source(File::with_name("Config.dev.toml").required(false))
            .add_source(Environment::with_prefix("TGCAPTCHA"))
            .build()?
            .try_deserialize()
    }

    pub fn get(&self, chat_id: &ChatId) -> &GroupSettings {
        match self.groups_settings.get(chat_id) {
            Some(s) => s,
            None => &self.fallback_group_settings,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GroupSettings {
    pub custom_chat_name: Option<String>,
    /// List of allowed admin to use bot commands and admin related stuff
    pub custom_admins: Option<Vec<UserId>>,
    #[serde(with = "humantime_serde")]
    pub ban_after: Duration,
    #[serde(skip_serializing_if = "MessagesText::is_default", default)]
    pub custom_messages: MessagesText
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            custom_chat_name: None,
            custom_admins: None,
            ban_after: Duration::from_secs(60 * 5),
            custom_messages: MessagesText::default()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct MessagesText {
    pub new_user_template: String,
    pub admin_approve: String,
    pub admin_only_error: String,
    pub user_doesnt_match_error: String,
    pub wrong_answer: String,
    pub admin_approved_user: String,
    pub correct_answer: String,
}

impl MessagesText {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn create_welcome_msg(&self, user: &User, chat_name: &str, question: Question) -> String {
        let msg = self
            .new_user_template
            .replace("{TAGUSER}", &user_mention_or_link(user))
            .replace("{CHATNAME}", chat_name);

        format!("{msg}\n<b>{question}</b>")
    }
}

impl Default for MessagesText {
    fn default() -> Self {
        Self {
            new_user_template: concat!(
                "Hello {TAGUSER}.\n",
                "Welcome to {CHATNAME} group.\n",
                "For accessing group, please choose the right answer from bellow options.",
            ).to_owned(),
            admin_approve: "Confirmation by admin ✅".to_owned(),
            admin_only_error: "❌ Only group admins can use this button".to_owned(),
            user_doesnt_match_error: "❌ This message isn't for you".to_owned(),
            wrong_answer: "❌ Your answer was wrong, you will be banned from group shortly".to_owned(),
            admin_approved_user: "✅ You approved this user".to_owned(),
            correct_answer: "✅ Your answer was correct! Now you can chat in the group".to_owned(),
        }
    }
}
