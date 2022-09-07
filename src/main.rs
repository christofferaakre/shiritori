use serenity::Client;
use shiritori::constants::DISCORD_BOT_TOKEN;
use shiritori::shiritori::Shiritori;

#[tokio::main]
async fn main() {
    let shiritori = Shiritori::new();
    let mut client = Client::builder(&DISCORD_BOT_TOKEN, shiritori.intents)
        .event_handler(shiritori)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
