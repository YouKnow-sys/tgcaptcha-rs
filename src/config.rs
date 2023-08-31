use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use teloxide::types::{ChatId, UserId};

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: ChatId,
    pub override_chat_name: Option<String>,
    pub admins: ChatAdmins,
    pub messages: MessagesText,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ChatAdmins {
    Explicit(Vec<UserId>),
    AllAdmins,
}

#[derive(Debug, Deserialize)]
pub struct MessagesText {
    /// Example:
    /// ```txt
    /// Hello {TAGUSER}.
    /// Welcome to {CHATNAME} group.
    /// For accessing group, please choose the right answer from bellow options.
    /// ```    
    pub new_user_template: String,
    pub admin_approve: String,
    pub admin_only_error: String,
    pub user_doesnt_match_error: String,
    pub wrong_answer: String,
    pub admin_approved_user: String,
    pub correct_answer: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub bot_token: String,
    pub allowed_chats: Vec<Chat>,
}

impl AppConfig {
    pub fn try_read() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("config.dev.toml").required(false))
            .add_source(Environment::with_prefix("TGCAPTCHA"))
            .build()?;

        s.try_deserialize()
    }
}
