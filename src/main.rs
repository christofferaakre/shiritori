use serenity::Client;
use shiritori::{Shiritori, DISCORD_BOT_TOKEN};

#[tokio::main]
async fn main() {
    let mut client = Client::builder(&DISCORD_BOT_TOKEN, Shiritori::intents())
        .event_handler(Shiritori)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
