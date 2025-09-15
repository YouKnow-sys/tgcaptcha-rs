use std::sync::Arc;
use std::time::Instant;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use teloxide::prelude::*;

mod commands;
mod config;
mod helpers;
mod join_check;

type HandlerResult = anyhow::Result<()>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Captcha bot...");

    let bot_start_time = Instant::now();

    let config = config::BotConfig::try_read().expect("Failed to read config");

    let pool = connect_database(&config.database_path)
        .await
        .expect("failed to connect to database");

    let bot = Bot::new(config.bot_token);

    tokio::spawn(join_check::storage::delete_expired_task(
        bot.clone(),
        pool.clone(),
    ));

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(Message::filter_new_chat_members().endpoint(join_check::join_handler))
                .branch(Message::filter_text().endpoint(commands::command_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(join_check::callback_handler));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .dependencies(dptree::deps![
            bot_start_time,
            Arc::new(config.groups_config),
            pool
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn connect_database(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
