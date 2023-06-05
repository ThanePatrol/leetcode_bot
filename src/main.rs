use discord::Discord;
use discord::model::{Channel, ChannelId, Event, ServerId};

fn main() {

    //permissions integer: 397553166416
    //test role id: 1115220490122960896
    //bot token: MTExNDY4MzA2MjQxNzExNzIxNA.GR-cBT.8NFWs_jcOL2WgyZIgbaEKxI4HRrxGhF8cHkG_c
    //discord url: https://discord.com/api/oauth2/authorize?client_id=1114683062417117214&permissions=326685952016&scope=bot
    let discord = Discord::from_bot_token("MTExNDY4MzA2MjQxNzExNzIxNA.GR-cBT.8NFWs_jcOL2WgyZIgbaEKxI4HRrxGhF8cHkG_c")
        .expect("login failed");



    let (mut connection, ready) = discord.connect().expect("connection failed");
    //discord.send_message(ChannelId(968378944472645646), "Hello world!", "true", false).expect("fs");

    for server in ready.servers.iter() {
        for members in discord.get_server_members(server.id()).iter() {
            println!("{:?}", members);
            for member in members.iter() {
                let message = format!("Hey, {:?}, ping!", member.roles);
                println!("{}", message);
            }

        }
    }

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(msg)) => {
                println!("{} says: {:?}", msg.author.name, msg);
            }
            _ => {}
        }
    }

    println!("Hello, world!");
}
