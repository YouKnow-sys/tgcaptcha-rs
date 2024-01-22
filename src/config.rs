use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use teloxide::{
    types::{ChatId, User, UserId},
    utils::html::{escape, link},
};

use crate::{crates::Crate, join_check::MathQuestion};

#[derive(Default, Deserialize)]
pub struct BotConfig {
    pub bot_token: String,
    #[serde(flatten, default)]
    pub groups_config: GroupsConfig,
}

impl BotConfig {
    pub fn try_read() -> Result<BotConfig, ConfigError> {
        Config::builder()
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("config.dev.toml").required(false))
            .add_source(Environment::with_prefix("TGCAPTCHA"))
            .build()?
            .try_deserialize()
    }
}

#[serde_as]
#[derive(Default, Deserialize)]
pub struct GroupsConfig {
    /// List of allowed groups, if `None` bot will allow all groups
    #[serde(default)]
    pub allowed_groups: Vec<ChatId>,
    #[serde(default)]
    fallback_group_settings: GroupSettings,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    #[serde(default)]
    groups_settings: HashMap<i64, GroupSettings>,
}

impl GroupsConfig {
    pub fn is_group_allowed(&self, chat_id: ChatId) -> bool {
        self.allowed_groups.is_empty() || self.allowed_groups.contains(&chat_id)
    }

    pub fn get(&self, chat_id: ChatId) -> &GroupSettings {
        match self.groups_settings.get(&chat_id.0) {
            Some(s) => s,
            None => &self.fallback_group_settings,
        }
    }
}

#[derive(Deserialize)]
pub struct GroupSettings {
    pub custom_chat_name: Option<String>,
    /// List of allowed admin to use bot commands and admin related stuff
    pub custom_admins: Option<Vec<UserId>>,
    #[serde(with = "humantime_serde")]
    pub ban_after: Duration,
    /// allow all users in group to use crates commands (crate and doc)
    #[serde(default)]
    pub crates_commands: bool,
    #[serde(default)]
    pub messages: MessagesText,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            custom_chat_name: None,
            custom_admins: None,
            ban_after: Duration::from_secs(60 * 5),
            crates_commands: false,
            messages: MessagesText::default(),
        }
    }
}

#[derive(Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct MessagesText {
    pub join_captcha: JoinCaptchaMessages,
    pub crates: CratesMessages,
}

#[derive(Deserialize, PartialEq)]
#[serde(default)]
pub struct JoinCaptchaMessages {
    new_user_template: String,
    pub admin_approve: String,
    pub admin_only_error: String,
    pub user_doesnt_match_error: String,
    pub wrong_answer: String,
    pub admin_approved_user: String,
    pub correct_answer: String,
    pub unauthorized_group: String,
}

impl JoinCaptchaMessages {
    pub fn create_welcome_msg(
        &self,
        user: &User,
        chat_name: &str,
        question: MathQuestion,
    ) -> String {
        let msg = self
            .new_user_template
            .replace("{TAGUSER}", &link(user.url().as_str(), &user.full_name()))
            .replace("{CHATNAME}", &escape(chat_name));

        format!("{msg}\n<b>{question}</b>")
    }
}

impl Default for JoinCaptchaMessages {
    fn default() -> Self {
        Self {
            new_user_template: concat!(
                "Hello {TAGUSER}.\n",
                "Welcome to {CHATNAME} group.\n",
                "For accessing group, please choose the right answer from bellow options.",
            )
            .to_owned(),
            admin_approve: "Confirmation by admin ✅".to_owned(),
            admin_only_error: "❌ Only group admins can use this button".to_owned(),
            user_doesnt_match_error: "❌ This message isn't for you".to_owned(),
            wrong_answer: "❌ Your answer was wrong, you will be banned from the group shortly"
                .to_owned(),
            admin_approved_user: "✅ You approved this user".to_owned(),
            correct_answer: "✅ Your answer was correct! Now you can chat in the group".to_owned(),
            unauthorized_group: "❌ This group isn't authorized. Goodbye!".to_owned(),
        }
    }
}

#[derive(Deserialize, PartialEq)]
#[serde(default)]
pub struct CratesMessages {
    crate_found_msg: String,
    crate_link_found: String,
    doc_found: String,
    pub crate_doc_button: String,
    pub crate_github_page: String,
    crate_not_found: String,
    crate_not_found_suggestion: String,
    invalid_query_in_doc_lookup: String,
    send_request_failed: String,
    crate_io_json_parse_error: String,
}

impl CratesMessages {
    pub fn crate_found_message(&self, crate_info: &Crate) -> String {
        // maybe use some formatting lib...
        let version = match crate_info
            .max_stable_version
            .as_ref()
            .or(crate_info.max_version.as_ref())
        {
            Some(v) => v.as_str(),
            None => "&lt;unknown version&gt;",
        };
        let description = match crate_info.description.as_ref() {
            Some(d) => d.as_str(),
            None => "<i>&lt;no description available&gt;</i>",
        };
        let updated_at = DateTime::parse_from_rfc3339(&crate_info.updated_at)
            .unwrap_or(Utc::now().into())
            .with_timezone(&Utc);

        self.crate_found_msg
            .replace("{NAME}", &crate_info.name)
            .replace("{VERSION}", version)
            .replace("{DESCRIPTION}", &escape(description))
            .replace("{DOWNLOADS}", &crate_info.downloads.to_string())
            .replace(
                "{UPDATED_AT}",
                &updated_at.format("%Y/%m/%d %H:%M:%S").to_string(),
            )
    }

    pub fn crate_link_found_message(&self, name: &str) -> String {
        self.crate_link_found.replace("{NAME}", &escape(name))
    }

    pub fn doc_found_message(&self, name: &str, query: &str) -> String {
        self.doc_found
            .replace("{NAME}", name)
            .replace("{QUERY}", query)
    }

    pub fn crate_not_found_message(&self, crate_name: &str, suggestion: Option<String>) -> String {
        match suggestion {
            Some(s) => self
                .crate_not_found_suggestion
                .replace("{CRATE_NAME}", crate_name)
                .replace("{CRATE_NAME_MEAN}", &s),
            None => self.crate_not_found.replace("{CRATE_NAME}", crate_name),
        }
    }

    pub fn invalid_query_in_doc_lookup_message(&self, query: &str) -> String {
        self.invalid_query_in_doc_lookup.replace("{QUERY}", query)
    }

    pub fn send_request_failed_message(&self, error: String) -> String {
        self.send_request_failed.replace("{DETAIL}", &error)
    }

    pub fn crate_io_json_parse_error_message(&self, error: String) -> String {
        self.crate_io_json_parse_error.replace("{DETAIL}", &error)
    }
}

impl Default for CratesMessages {
    fn default() -> Self {
        Self {
            crate_found_msg: concat!(
                "<b>Crate:</b> {NAME}\n",
                "<b>Version:</b> <code>{VERSION}</code>\n",
                "<b>Description:</b> {DESCRIPTION}\n",
                "<b>Downloads:</b> <code>{DOWNLOADS}</code>\n",
                "<b>Updated at:</b> {UPDATED_AT}",
            ).to_owned(),
            crate_link_found: "<b>Crate:</b> {NAME}".to_owned(),
            doc_found: concat!(
                "<b>Crate:</b> {NAME}\n",
                "<b>Input query:</b> {QUERY}"
            ).to_owned(),
            crate_doc_button: "Open documentation".to_owned(),
            crate_github_page: "Open repository".to_owned(),
            crate_not_found: "❌ Crate <code>{CRATE_NAME}</code> not found".to_owned(),
            crate_not_found_suggestion: "❌ Crate <code>{CRATE_NAME}</code> not found. Did you mean <code>{CRATE_NAME_MEAN}</code>?".to_owned(),
            invalid_query_in_doc_lookup: "❌ Can't fine the first path element inside the query: {QUERY}".to_owned(),
            send_request_failed: "❌ Error when sending request for fetching crate information\n<pre><code class=\"language-error\">{DETAIL}</code></pre>".to_owned(),
            crate_io_json_parse_error: "❌ Cannot parse crates.io JSON response\n<pre><code class=\"language-error\">{DETAIL}</code></pre>".to_owned(),
        }
    }
}
