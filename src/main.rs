mod scrapers;
mod leetcode;
mod db_api;

use std::thread;
use std::time::Duration;
use discord::Discord;
use discord::model::{Channel, ChannelId, Event, ServerId};
use tokio::join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let env_file = dotenvy::dotenv().expect("Could not read .env file");
    let bot_token = std::env::var("BOT_TOKEN").expect("Error reading token from .env");

    let db_url = std::env::var("DATABASE_URL").expect("Error reading db url from .env");
    let pool = db_api::init_db(&db_url).await?;


    //
    // let discord = Discord::from_bot_token(&bot_token)
    //     .expect("login failed");
    //
    //
    // let (mut connection, ready) = discord.connect().expect("connection failed");
    //
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
    let mut driver_process = scrapers::init_webdriver();
    thread::sleep(Duration::from_millis(500));
    let questions = scrapers::scrape_neetcode().await?;
    println!("{:?}", questions);
    db_api::add_leetcode_entries_to_db(questions, &pool).await?;
    join!();

    driver_process.kill()?;
    Ok(())
}

fn ping_with_daily(channel_id: u64, role_id: u64, link: &str, client: &Discord) -> Result<(), Box<dyn std::error::Error>> {
    let msg = format!("<@&{}> The daily question is {}", role_id, link);
    client.send_message(
        ChannelId(channel_id),
        &*msg,
        "",
        false,
    )?;

    Ok(())
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

