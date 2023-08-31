use std::sync::Arc;

use teloxide::prelude::*;

mod commands;
mod config;
mod join_captcha;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Captcha bot...");

    let config = config::AppConfig::try_read().expect("Failed to read config");

    let bot = Bot::new(config.bot_token.clone());

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(Message::filter_new_chat_members().endpoint(join_captcha::join_handler))
                .branch(Message::filter_text().endpoint(commands::command_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(join_captcha::callback_handler));

    let mut dependency_map = DependencyMap::new();
    dependency_map.insert(Arc::new(config));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .dependencies(dependency_map)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
