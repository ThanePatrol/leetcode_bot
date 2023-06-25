mod scrapers;
mod leetcode;
mod db_api;
mod utils;
mod discord_api;

use std::collections::HashSet;
use std::error::Error;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use discord::{Discord, GetMessages};
use discord::model::{ChannelId, Event, Message, ServerId};
use crate::discord_api::{CommandType, DiscordAPI, QuestionQueue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("Could not read .env file");
    let bot_token = std::env::var("BOT_TOKEN").expect("Error reading token from .env");

    let db_url = std::env::var("DATABASE_URL").expect("Error reading db url from .env");
    let pool = Rc::new(db_api::init_db(&db_url).await?);


    let discord = Discord::from_bot_token(&bot_token)
        .expect("login failed");
    let discord = Rc::new(discord);

    let (_, _) = discord.connect()
        .expect("connection failed");

    let command_channel = std::env::var("COMMAND_CHANNEL_ID")
        .expect("Error reading command channel from .env")
        .parse::<u64>()
        .unwrap();

    let question_channel = std::env::var("QUESTION_CHANNEL_ID")
        .expect("Error reading question channel from .env")
        .parse::<u64>()
        .unwrap();

    let easy_id = std::env::var("EASY_ROLE_ID")
        .expect("Error reading role id from .env")
        .parse::<u64>()
        .unwrap();

    let med_id = std::env::var("MED_ROLE_ID")
        .expect("Error reading role id from .env")
        .parse::<u64>()
        .unwrap();

    let hard_id = std::env::var("HARD_ROLE_ID")
        .expect("Error reading role id from .env")
        .parse::<u64>()
        .unwrap();

    let bot_id = std::env::var("BOT_USER_ID")
        .expect("Error reading bot id from .env")
        .parse::<u64>()
        .unwrap();

    let api = discord_api::DiscordAPI::new(
        discord.clone(),
        command_channel,
        question_channel,
        easy_id,
        med_id,
        hard_id,
        bot_id
    );


    let mut question_queue = QuestionQueue::new(pool);
    let mut seen_commands = HashSet::new();



    loop {
        // check for command
        if let Ok(mut command) = discord.as_ref().get_messages(
            ChannelId(api.command_channel_id), GetMessages::MostRecent, Some(1)) {
            let mut cmd = String::new();
            match command.pop() {
                None => {continue}
                Some(c) => {
                    if seen_commands.contains(&c.id) {
                        continue
                    }
                    seen_commands.insert(c.id);
                    if c.author.bot {continue}
                    cmd = c.content;
                }
            }

            // parse command
            match DiscordAPI::parse_command(&cmd) {
                Err(e) => {api.send_error_message(Box::new(e))}
                Ok(action) => {
                    match action {
                        CommandType::AddQuestion => {
                            match api.add_question_to_queue(&cmd, &mut question_queue).await {
                                Ok(_) => { api.send_confirmation_message("Question added to queue :)") }
                                Err(e) => { api.send_error_message(e) }
                            };
                        }
                        CommandType::PostQuestion => {
                            match api.ping_with_daily(&mut question_queue).await {
                                Ok(_) => {api.send_confirmation_message("Pinged people :)") }
                                Err(e) => { api.send_error_message(e)}
                            }
                        }
                        CommandType::ViewQuestions => {
                            match api.get_all_questions_in_queue(&mut question_queue).await {
                                Ok(_) => {}
                                Err(e) => {api.send_error_message(e)}
                            }
                        }
                    }
                }
            }
        } else {
            thread::sleep(Duration::from_secs(1));
        }
        thread::sleep(Duration::from_millis(1500));
    }
}


