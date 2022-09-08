use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::bot::Bot;
use crate::constants::PREFIX;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Shiritori {
    bots: Mutex<HashMap<GuildId, Bot>>,
    pub intents: GatewayIntents,
}

impl Shiritori {
    pub fn new() -> Self {
        Self {
            bots: Mutex::new(HashMap::new()),
            intents: GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::GUILD_PRESENCES,
        }
    }
}

#[async_trait]
impl EventHandler for Shiritori {
    async fn message(&self, ctx: Context, message: Message) {
        // Check that message was not sent by the bot
        if ctx.cache.current_user().tag() == message.author.tag() {
            return;
        }

        let id = message.guild_id.expect("Failed to get guild id of message");

        let msg = message.content.clone();

        let split: Vec<&str> = msg.split_whitespace().collect();
        // split[0] should always exist because the only way to get an empty split
        // is splitting on an empty string, and you can't send empty strings on Discord.
        assert!(split.len() > 0, "Somehow the length of the split was 0??");
        if split[0] != PREFIX {
            println!("Message {} did not start with prefix {}.", msg, PREFIX);
            return;
        }

        // If no bot exists for this guild yet, create one.
        // Else, retrieve the correct bot from the hashmap
        let mut bots = self.bots.lock().await;
        let bot = match bots.entry(id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Bot::new()),
        };

        match split.len() {
            1 => bot.help(ctx, message).await,
            2 => match split[1] {
                "help" => bot.help(ctx, message).await,
                "history" => bot.history(ctx, message).await,
                "word" => bot.show_previous_word(ctx, message).await,
                _ => bot.play(ctx, message, split[1]).await,
            },
            _ => bot.not_recognised(ctx, message).await,
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
