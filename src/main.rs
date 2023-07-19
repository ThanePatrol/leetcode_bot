pub mod scrapers;
mod leetcode;
mod db_api;
mod utils;
mod discord_api;

use std::error::Error;
use std::ops::Sub;
use std::rc::Rc;
use std::time::{Duration};
use discord::{Discord};
use discord::model::{ChannelId, Event};
use time::macros::{format_description};
use time::{OffsetDateTime, Time};
use crate::discord_api::{CommandType, DiscordAPI, QuestionQueue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for reading a local .env, ie not in a docker container
    if let Ok(path) = dotenvy::dotenv() {
        println!(".env read at {:?}", path)
    }

    let bot_token = std::env::var("BOT_TOKEN").expect("Error reading token from .env");

    let db_url = std::env::var("DATABASE_URL").expect("Error reading db url from .env");
    let pool = Rc::new(db_api::init_db(&db_url).await?);


    let discord = Discord::from_bot_token(&bot_token)
        .expect("login failed");
    let discord = Rc::new(discord);

    let (mut connection, _) = discord.connect()
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

    let announcement_text = std::env::var("ANNOUNCEMENT_TEXT")
        .expect("Error reading announcement text from .env");

    let posting_time = std::env::var("TIME_TO_POST")
        .expect("Error reading posting time from .env");

    let format = format_description!("[hour]:[minute]:[second]");
    println!("{}", posting_time);
    let posting_time = Time::parse(&*posting_time, &format)
        .expect("Error parsing time from .env");


    let mut time_at_last_ping = OffsetDateTime::now_utc().sub(Duration::from_secs(86400));

    let api = DiscordAPI::new(
        discord.clone(),
        command_channel,
        question_channel,
        easy_id,
        med_id,
        hard_id,
        bot_id,
        announcement_text,
    );

    let mut question_queue = QuestionQueue::new(pool);

    println!("entering main loop");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                //ignore bot messages or messages not in the command channel
                if message.author.bot  || message.channel_id != ChannelId(command_channel) {
                    continue;
                }
                let cmd = message.content;

                match DiscordAPI::parse_command(&cmd) {
                    Err(e) => { api.send_error_message(Box::new(e)) }
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
                                    Ok(_) => { api.send_confirmation_message("Pinged people :)") }
                                    Err(e) => { api.send_error_message(e) }
                                }
                            }
                            CommandType::ViewQuestions => {
                                match api.get_all_questions_in_queue(&mut question_queue).await {
                                    Ok(_) => {}
                                    Err(e) => { api.send_error_message(e) }
                                }
                            }
                        }
                    }
                }
            }
            Err(discord::Error::Closed(code, body)) => {
                println!("connection dropped with code: {:?} and {} \n reconnecting now...", code, body);
                let (conn, _) = discord.connect()
                    .expect("Connection failed when trying to reconnect");
                connection = conn;
            }
            _ => {
                // check if it's the time of day to make a post
                let duration_since_last_ping = OffsetDateTime::now_utc() - time_at_last_ping;
                let now_time = OffsetDateTime::now_utc().time();

                // check if the current time is within 5 minutes of the posting window
                let is_within_posting_window = (now_time - posting_time).whole_minutes().abs() < 5;
                if duration_since_last_ping.whole_hours() >= 24 && is_within_posting_window {
                    let possible_fail = api.ping_with_daily(&mut question_queue).await;

                    match possible_fail {
                        Ok(_) => {},
                        Err(e) => println!("failed to post error {:?} at {:?}", e, OffsetDateTime::now_utc())
                    };
                    time_at_last_ping = OffsetDateTime::now_utc();
                }

                // a simple request to keep the connection alive
                discord.get_messages()

            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_checks_are_accurate() {
        let posting_time = OffsetDateTime::now_utc().time();
        let time_at_last_ping = OffsetDateTime::now_utc()
            .sub(Duration::from_secs(86400));

        let duration_since_last_ping = OffsetDateTime::now_utc() - time_at_last_ping;
        assert!(duration_since_last_ping.whole_hours() <= 24);

        let now_time = OffsetDateTime::now_utc().time();

        // check if the current time is within 5 minutes of the posting window
        let is_within_posting_window = (now_time - posting_time).whole_minutes() < 5;

        assert!(is_within_posting_window);
    }
}

