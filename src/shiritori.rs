use indoc::formatdoc;
use kanji::Character;
use kanji::Hiragana;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::bot::Bot;
use crate::constants::{HELP_STRING, PREFIX};
use crate::word::Word;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

type ShardID = u64;

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
        let id = message.guild_id.expect("Failed to get guild id of message");

        // If no bot exists for this guild yet, create one

        let bot = match self.bots.lock().await.entry(id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Bot::new()),
        };

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
            1 => {
                self.bots
                    .lock()
                    .await
                    .get(&id)
                    .unwrap()
                    .help(ctx, message)
                    .await
            }
            2 => match split[1] {
                "help" => {
                    self.bots
                        .lock()
                        .await
                        .get(&id)
                        .unwrap()
                        .help(ctx, message)
                        .await
                }
                "history" => {
                    self.bots
                        .lock()
                        .await
                        .get(&id)
                        .unwrap()
                        .history(ctx, message)
                        .await
                }
                "word" => {
                    self.bots
                        .lock()
                        .await
                        .get(&id)
                        .unwrap()
                        .show_previous_word(ctx, message)
                        .await
                }
                _ => {
                    self.bots
                        .lock()
                        .await
                        .get(&id)
                        .unwrap()
                        .play(ctx, message, split[1])
                        .await
                }
            },
            _ => {
                self.bots
                    .lock()
                    .await
                    .get(&id)
                    .unwrap()
                    .not_recognised(ctx, message)
                    .await
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
