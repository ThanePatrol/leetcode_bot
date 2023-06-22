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
use discord::model::{Channel, ChannelId, Event, Message, ServerId};
use tokio::join;
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

    let (mut connection, ready) = discord.connect()
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


    // //discord.edit_member_roles() can be used to assign a role to a user
    //
    // for server in ready.servers.iter() {
    //     for members in discord.get_server_members(server.id()).iter() {
    //         println!("{:?}", members);
    //         for member in members.iter() {
    //
    //             ping_with_daily(
    //                 968378944472645647,
    //                 1115220490122960896,
    //                 "https://leetcode.com/problems/minimum-flips-to-make-a-or-b-equal-to-c/",
    //                 &discord
    //             )
    //                 .expect("Error pinging with daily");
    //         }
    //     }
    // }

    // listen for commands

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
                    }
                }
            }
        } else {
            thread::sleep(Duration::from_secs(1));
        }
        thread::sleep(Duration::from_millis(1500));


    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //assumes chromedriver is already running and the total amount of neetcode questions is 434
    #[tokio::test]
    async fn test_all_questions_scraped_from_neetcode() {
        let env_file = dotenvy::dotenv().expect("Could not read .env file");
        let driver = scrapers::init_webdriver();
        let questions = scrapers::scrape_neetcode().await.unwrap();
        assert_eq!(questions.len(), 434);
    }
}

