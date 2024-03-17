# Rust Telegram Captcha Bot
[![Build Status](https://github.com/YouKnow-sys/tgcaptcha-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/YouKnow-sys/tgcaptcha-rs/actions?workflow=Rust%20CI)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/YouKnow-sys/tgcaptcha-rs/blob/master/LICENSE)

A Telegram bot that validates new users that enter to the (**super**)group with a simple math question.

Bot written fully in Rust 🦀.

## How to build and use:
1. Clone the bot and build it using `cargo build --release`.
2. Before running the bot, set the `bot_token` in the `Config.toml` file
   or use the `TGCAPTCHA_BOT_TOKEN` environment value.
3. Customize the `Config.toml` file according to your preferences. 
   You can modify various aspects of the bot,
   such as restricting its functionality to specific groups
   or setting custom settings and messages for different groups.
4. Alternatively, you can store all the settings in the environment instead of using `Config.toml`.
   We utilize the [Config crate](https://crates.io/crates/config/) for managing our configuration.
   Refer to its documentation for more details.
5. Add the bot to any desired group, ensuring it has administrator privileges.
6. The bot will restrict new users from sending messages in the group
   until they answer a math question sent by the bot.
   If it takes too long for the user to respond or choose the wrong answer, they will be banned.

## Commands:
- `/help`: show commands help message.
- `/status`: check the bot status, can be used to make sure that the bot is up and running.
- `/uptime`: show the bot uptime.
- `/sourcecode`: share a link to the source code of the bot.
