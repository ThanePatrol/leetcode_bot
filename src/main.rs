use discord::Discord;
use discord::model::{Channel, ChannelId, Event, ServerId};

fn main() {
    let env_file = dotenvy::dotenv().expect("Could not read .env file");
    let bot_token = std::env::var("BOT_TOKEN").expect("Error reading token from .env");

    //permissions integer: 397553166416
    //test role id: 1115220490122960896
    //bot token: MTExNDY4MzA2MjQxNzExNzIxNA.GR-cBT.8NFWs_jcOL2WgyZIgbaEKxI4HRrxGhF8cHkG_c
    //discord url: https://discord.com/api/oauth2/authorize?client_id=1114683062417117214&permissions=326685952016&scope=bot
    let discord = Discord::from_bot_token(&bot_token)
        .expect("login failed");


    let (mut connection, ready) = discord.connect().expect("connection failed");
    //discord.send_message(ChannelId(968378944472645646), "Hello world!", "true", false).expect("fs");

    //discord.edit_member_roles() can be used to assign a role to a user

    for server in ready.servers.iter() {
        for members in discord.get_server_members(server.id()).iter() {
            println!("{:?}", members);
            for member in members.iter() {
                //discord.broadcast_typing(ChannelId(968378944472645647), )
                //discord.send_message(ChannelId(968378944472645647), "<@&1115220490122960896>leetcode test", "", false).expect("errpr");

                ping_with_daily(
                    968378944472645647,
                    1115220490122960896,
                    "https://leetcode.com/problems/minimum-flips-to-make-a-or-b-equal-to-c/",
                    &discord
                )
                    .expect("Error pinging with daily");

                // let message = format!("Hey, {:?}, ping!", member.roles);
                // println!("{}", message);
            }
        }
    }

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

