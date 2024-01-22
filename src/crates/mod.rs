use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, InlineQueryResult,
        InlineQueryResultArticle, InputMessageContent, InputMessageContentText, ParseMode,
    },
    utils::html,
    Bot,
};

use crate::config::GroupSettings;

use error::CratesError;
pub use internals::{get_documentation, Crate};

mod error;
mod internals;

pub enum CratesAction {
    CrateLookup,
    DocLookup,
}

pub async fn crates_command_handler(
    bot: Bot,
    msg: Message,
    client: reqwest::Client,
    group_settings: &GroupSettings,
    query: &str,
    action: CratesAction,
) -> Result<()> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(());
    }

    match action {
        CratesAction::CrateLookup => {
            let crate_variant = match internals::crate_lookup(&client, query).await {
                Ok(crate_info) => crate_info,
                Err(error) => {
                    let message = get_message(error, query, group_settings);
                    bot.send_message(msg.chat.id, message)
                        .reply_to_message_id(msg.id)
                        .parse_mode(ParseMode::Html)
                        .await?;

                    return Ok(());
                }
            };

            match crate_variant {
                internals::CrateVariant::Crate(c) => {
                    bot.send_message(
                        msg.chat.id,
                        group_settings.messages.crates.crate_found_message(&c),
                    )
                    .reply_to_message_id(msg.id)
                    .parse_mode(ParseMode::Html)
                    .reply_markup({
                        let keyboard =
                            InlineKeyboardMarkup::new([vec![InlineKeyboardButton::new(
                                &group_settings.messages.crates.crate_doc_button,
                                InlineKeyboardButtonKind::Url(
                                    Url::parse(&get_documentation(&c)).expect("Invalid URL"),
                                ),
                            )]]);

                        match c.repository.map(|v| v.parse::<Url>()) {
                            Some(Ok(repo)) => keyboard.append_row(vec![InlineKeyboardButton::new(
                                &group_settings.messages.crates.crate_github_page,
                                InlineKeyboardButtonKind::Url(repo),
                            )]),
                            _ => keyboard,
                        }
                    })
                    .await?;
                }
                internals::CrateVariant::Link(l) => {
                    bot.send_message(
                        msg.chat.id,
                        group_settings
                            .messages
                            .crates
                            .crate_link_found_message(query),
                    )
                    .reply_to_message_id(msg.id)
                    .parse_mode(ParseMode::Html)
                    .reply_markup({
                        InlineKeyboardMarkup::new([vec![InlineKeyboardButton::new(
                            &group_settings.messages.crates.crate_doc_button,
                            InlineKeyboardButtonKind::Url(Url::parse(&l).expect("Invalid URL")),
                        )]])
                    })
                    .await?;
                }
            }
        }
        CratesAction::DocLookup => {
            let doc = match internals::doc_lookup(&client, query).await {
                Ok(d) => d,
                Err(error) => {
                    let query = if matches!(error, CratesError::InvalidQueryInDocLookup) {
                        query
                    } else {
                        query.split_once("::").unwrap().0
                    };
                    let message = get_message(error, query, group_settings);
                    bot.send_message(msg.chat.id, message)
                        .reply_to_message_id(msg.id)
                        .parse_mode(ParseMode::Html)
                        .await?;

                    return Ok(());
                }
            };

            // unwrap in here is ok
            let (crate_name, _) = query.split_once("::").unwrap();

            bot.send_message(
                msg.chat.id,
                group_settings
                    .messages
                    .crates
                    .doc_found_message(crate_name, query),
            )
            .reply_to_message_id(msg.id)
            .parse_mode(ParseMode::Html)
            .reply_markup({
                InlineKeyboardMarkup::new([vec![InlineKeyboardButton::new(
                    &group_settings.messages.crates.crate_doc_button,
                    InlineKeyboardButtonKind::Url(Url::parse(&doc).expect("Invalid URL")),
                )]])
            })
            .await?;
        }
    }

    Ok(())
}

pub async fn inline_query_handler(bot: Bot, q: InlineQuery, client: reqwest::Client) -> Result<()> {
    let query = q.query.trim().to_lowercase();

    if query.is_empty() {
        let help = InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                "help".to_string(),
                "Enter crate name",
                InputMessageContent::Text(
                    InputMessageContentText::new("Enter any <b>crate</b> name for search")
                        .parse_mode(ParseMode::Html),
                ),
            )
            .description("Enter any crate name for search")
            .reply_markup(InlineKeyboardMarkup::new([vec![
                InlineKeyboardButton::new(
                    "Try now!",
                    InlineKeyboardButtonKind::SwitchInlineQuery("log".to_owned()),
                ),
            ]])),
        );

        bot.answer_inline_query(&q.id, [help]).send().await?;

        return Ok(());
    }

    let crates: Vec<_> = internals::matching_crates(&client, &q.query)
        .await
        .collect();

    if crates.is_empty() {
        let no_crate_found = InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                "no-crate-matched".to_string(),
                "Nothing matched",
                InputMessageContent::Text(
                    InputMessageContentText::new(format!(
                        "Can't find any crate that match \"<code>{}</code>\".",
                        html::escape(&q.query)
                    ))
                    .parse_mode(ParseMode::Html),
                ),
            )
            .description("No matching crate found"),
        );

        bot.answer_inline_query(&q.id, [no_crate_found])
            .send()
            .await?;
    } else {
        let crates = crates.into_iter().map(|c| {
            let version = match c.max_stable_version.as_ref().or(c.max_version.as_ref()) {
                Some(v) => v.as_str(),
                None => "&lt;unknown version&gt;",
            };
            let description = match c.description.as_ref() {
                Some(d) => d.as_str(),
                None => "<i>&lt;no description available&gt;</i>",
            };
            let updated_at = DateTime::parse_from_rfc3339(&c.updated_at)
                .unwrap_or(Utc::now().into())
                .with_timezone(&Utc);
            InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    &c.name,
                    &c.name,
                    InputMessageContent::Text(
                        InputMessageContentText::new(format!(
                            concat!(
                                "<b>Crate:</b> {name}\n",
                                "<b>Version:</b> <code>{version}</code>\n",
                                "<b>Description:</b> {desc}\n",
                                "<b>Downloads:</b> <code>{downloads}</code>\n",
                                "<b>Updated at:</b> {up_at}",
                            ),
                            name = c.name,
                            version = version,
                            desc = html::escape(description),
                            downloads = c.downloads,
                            up_at = updated_at.format("%Y/%m/%d %H:%M:%S"),
                        ))
                        .parse_mode(ParseMode::Html),
                    ),
                )
                .description(description)
                .reply_markup({
                    let keyboard = InlineKeyboardMarkup::new([vec![InlineKeyboardButton::new(
                        "Open documentation",
                        InlineKeyboardButtonKind::Url(
                            Url::parse(&get_documentation(&c)).expect("Invalid URL"),
                        ),
                    )]]);

                    match c.repository.map(|v| v.parse::<Url>()) {
                        Some(Ok(repo)) => keyboard.append_row(vec![InlineKeyboardButton::new(
                            "Open repository",
                            InlineKeyboardButtonKind::Url(repo),
                        )]),
                        _ => keyboard,
                    }
                }),
            )
        });

        bot.answer_inline_query(&q.id, crates).send().await?;
    }

    Ok(())
}

fn get_message(error: CratesError, query: &str, group_settings: &GroupSettings) -> String {
    match error {
        error::CratesError::Reqwest(e) => group_settings
            .messages
            .crates
            .send_request_failed_message(e.to_string()),
        error::CratesError::CrateNotFound(suggestion) => group_settings
            .messages
            .crates
            .crate_not_found_message(query, suggestion),
        error::CratesError::InvalidQueryInDocLookup => group_settings
            .messages
            .crates
            .invalid_query_in_doc_lookup_message(query),
        error::CratesError::CratesIoParseJson(e) => group_settings
            .messages
            .crates
            .crate_io_json_parse_error_message(e.to_string()),
    }
}
