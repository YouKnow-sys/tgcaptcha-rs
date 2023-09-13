use std::sync::Arc;

use dashmap::DashMap;
use join_check::MathQuestion;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, types::MessageId};

mod commands;
mod config;
mod join_check;

type HandlerError = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), HandlerError>;
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

    let config = config::BotConfig::try_read().expect("Failed to read config");

    let bot = Bot::new(config.bot_token);

    let handler = dptree::entry()
        .enter_dialogue::<Update, InMemStorage<DialogueDataType>, DialogueDataType>()
        .branch(
            Update::filter_message()
                .branch(Message::filter_new_chat_members().endpoint(join_check::join_handler))
                .branch(Message::filter_text().endpoint(commands::command_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(join_check::callback_handler));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .dependencies(dptree::deps![
            Arc::new(config.groups_config),
            InMemStorage::<DialogueDataType>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
