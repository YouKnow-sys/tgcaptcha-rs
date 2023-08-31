use std::sync::Arc;

use dashmap::DashMap;
use join_check::Question;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, types::MessageId};

mod commands;
mod config;
mod join_check;

type HandlerError = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), HandlerError>;
type DaialogueDataType = Arc<DashMap<MessageId, DialogueData>>;
type GroupDialogue = Dialogue<DaialogueDataType, InMemStorage<DaialogueDataType>>;

#[derive(Clone)]
pub struct DialogueData {
    user_id: UserId,
    question: Question,
    passed: bool,
}

impl DialogueData {
    fn new(user_id: UserId, question: Question) -> Self {
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

    let bot = Bot::new(
        std::env::var("TGCAPTCHA_BOT_TOKEN")
            .expect("Can't find the TGCAPTCHA_BOT_TOKEN environment variable with the token."),
    );

    let config = Arc::new(config::AppConfig::try_read().expect("Failed to read config"));

    let handler = dptree::entry()
        .enter_dialogue::<Update, InMemStorage<DaialogueDataType>, DaialogueDataType>()
        .branch(
            Update::filter_message()
                .branch(Message::filter_new_chat_members().endpoint(join_check::join_handler))
                .branch(Message::filter_text().endpoint(commands::command_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(join_check::callback_handler));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .dependencies(dptree::deps![
            config,
            InMemStorage::<DaialogueDataType>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
