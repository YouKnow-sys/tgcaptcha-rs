use teloxide::prelude::*;

mod commands;
mod join_captcha;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Captcha bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(Message::filter_new_chat_members().endpoint(join_captcha::join_handler))
                .branch(Message::filter_text().endpoint(commands::command_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(join_captcha::callback_handler));

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
