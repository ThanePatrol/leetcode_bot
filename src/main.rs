mod scrapers;

use discord::Discord;
use discord::model::{Channel, ChannelId, Event, ServerId};
use crate::scrapers::{init_webdriver, scrape_neetcode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let env_file = dotenvy::dotenv().expect("Could not read .env file");
    let bot_token = std::env::var("BOT_TOKEN").expect("Error reading token from .env");
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
    //init_webdriver();
    println!("here");
    scrape_neetcode().await?;

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


