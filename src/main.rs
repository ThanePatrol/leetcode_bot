mod scrapers;
mod leetcode;
mod db_api;
mod utils;
mod discord_api;

use std::collections::HashSet;
use std::ops::Sub;
use std::rc::Rc;
use std::thread;
use std::time::{Duration};
use discord::{Discord, GetMessages};
use discord::model::{ChannelId, Message, ServerId};
use time::macros::{format_description};
use time::{OffsetDateTime, Time};
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

    let announcement_text = std::env::var("ANNOUNCEMENT_TEXT")
        .expect("Error reading announcement text from .env");

    let posting_time = std::env::var("TIME_TO_POST")
        .expect("Error reading posting time from .env");

    let format = format_description!("[hour]:[minute]:[second]");
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

    // make sure we don't respond multiple times to the same question
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

            // check if it's the time of day to make a post
            let duration_since_last_ping = OffsetDateTime::now_utc() - time_at_last_ping;
            let now_time = OffsetDateTime::now_utc().time();

            // check if the current time is within 5 minutes of the posting window
            let is_within_posting_window = (now_time - posting_time).whole_minutes() < 5;
            if duration_since_last_ping.whole_hours() <= 24 && is_within_posting_window {
                api.ping_with_daily(&mut question_queue).await?;
                time_at_last_ping = OffsetDateTime::now_utc();
            }

        }
        thread::sleep(Duration::from_secs(1));
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

