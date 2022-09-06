use kanji::Character;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct Shiritori;

use indoc::indoc;

const PREFIX: &str = "!shiri";

const HELP_STRING: &str = indoc! {r#"
    Play shiritori!
    Commands:
        !shiri / !shiri help - Display this help message
"#};

impl Shiritori {
    pub fn intents() -> GatewayIntents {
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_PRESENCES
    }

    async fn help(ctx: Context, message: Message) {
        if let Err(why) = message.channel_id.say(&ctx.http, HELP_STRING).await {
            println!("Error sending message: {:?}", why);
        }
    }

    pub async fn not_recognised(ctx: Context, message: Message) {
        let error_message = format!("{} is not a recognised command", message.content.clone());
        println!("{}", error_message);
        if let Err(why) = message.channel_id.say(&ctx.http, error_message).await {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn play(ctx: Context, message: Message, word: &str) {
        let characters = word.chars().map(Character::new);
        // Kind of an ugly way to do this, but it works
        let all_hiragana = characters.clone().all(|c| {
            if let Character::Hiragana(_) = c {
                return true;
            }
            false
        });

        if !all_hiragana {
            println!("Word {} contained non-hiragana characters", word);
            return;
        }

        // #TODO: Check that the word is playable
        // #TODO: Check if the word ends in ん
        // If all the characters are hiragana, play the word.

        let played_message = format!("{} Played word: {}", message.author.mention(), word);
        message
            .channel_id
            .say(&ctx.http, played_message)
            .await
            .unwrap();
    }
}

pub const DISCORD_BOT_TOKEN: &str = include_str!("../DISCORD_BOT_TOKEN");

#[async_trait]
impl EventHandler for Shiritori {
    async fn message(&self, ctx: Context, message: Message) {
        let msg = message.content.clone();
        let split: Vec<&str> = msg.split_whitespace().collect();
        // split[0] should always exist because the only way to get an empty split
        // is splitting on an empty string, and you can't send empty strings on Discord.
        assert!(split.len() > 0, "Somehow the length of the split was 0??");
        if split[0] != PREFIX {
            println!("Message {} did not start with prefix {}.", msg, PREFIX);
            return;
        }

        match split.len() {
            1 => Shiritori::help(ctx, message).await,
            2 => match split[1] {
                "help" => Shiritori::help(ctx, message).await,
                _ => Shiritori::play(ctx, message, split[1]).await,
            },
            _ => Shiritori::not_recognised(ctx, message).await,
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
