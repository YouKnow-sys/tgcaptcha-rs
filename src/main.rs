use std::{sync::Arc, time::Instant};

use dashmap::DashMap;
use join_check::MathQuestion;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, types::MessageId};

use commands::{admin_command_handler, user_command_handler, AdminCommands, UserCommands};

use crate::commands::{command_handler, Commands};

mod commands;
mod config;
mod crates;
mod join_check;
mod utils;

type DialogueDataType = Arc<DashMap<MessageId, DialogueData>>;
type GroupDialogue = Dialogue<DialogueDataType, InMemStorage<DialogueDataType>>;

#[derive(Clone)]
pub struct DialogueData {
    user_id: UserId,
    question: MathQuestion,
    passed: bool,
}

impl DialogueData {
    fn new(user_id: UserId, question: MathQuestion) -> Self {
        Self {
            user_id,
            question,
            passed: false,
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Captcha bot...");

    let bot_start_time = Instant::now();

    let config = config::BotConfig::try_read().expect("Failed to read config");

    let bot = Bot::new(config.bot_token);

    let client = reqwest::Client::new();

    let handler = dptree::entry()
        .branch(Update::filter_inline_query().endpoint(crates::inline_query_handler))
        .enter_dialogue::<Update, InMemStorage<DialogueDataType>, DialogueDataType>()
        .branch(
            Update::filter_message()
                .branch(Message::filter_new_chat_members().endpoint(join_check::join_handler))
                .branch(
                    dptree::entry()
                        .filter_command::<Commands>()
                        .endpoint(command_handler),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<AdminCommands>()
                        .endpoint(admin_command_handler),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<UserCommands>()
                        .endpoint(user_command_handler),
                ),
        )
        .branch(Update::filter_callback_query().endpoint(join_check::callback_handler));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .dependencies(dptree::deps![
            bot_start_time,
            client,
            Arc::new(config.groups_config),
            InMemStorage::<DialogueDataType>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
