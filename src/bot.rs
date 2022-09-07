use crate::word::Word;
use kanji::Character;
use kanji::Hiragana;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::*;
use serenity::prelude::*;

type Words = Mutex<Vec<Word>>;
pub struct Shiritori {
    words: Words,
    pub intents: GatewayIntents,
}

use indoc::{formatdoc};

use crate::constants::{HELP_STRING, PREFIX};

impl Shiritori {
    pub fn new() -> Self {
        Self {
            words: Mutex::new(vec![]),
            intents: GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::GUILD_PRESENCES,
        }
    }

    async fn help(&self, ctx: Context, message: Message) {
        if let Err(why) = message.channel_id.say(&ctx.http, HELP_STRING).await {
            println!("Error sending message: {:?}", why);
        }
    }

    pub async fn not_recognised(&self, ctx: Context, message: Message) {
        let error_message = format!("{} is not a recognised command", message.content.clone());
        println!("{}", error_message);
        if let Err(why) = message.channel_id.say(&ctx.http, error_message).await {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn play(&self, ctx: Context, message: Message, word: &str) {
        let channel = message.channel_id;
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

        let characters: Vec<Hiragana> = characters
            .map(|c| Hiragana::new(c.get()).unwrap())
            .collect();

        let last_character = characters
            .last()
            .expect("Failed to get last character of word");

        // #TODO: End game, update leaderbord, etc.
        if last_character == &Hiragana::new('ん').unwrap() {
            let fail_string = format!(
                "{} Your word {} ends in ん. Better luck next time!",
                message.author.mention(),
                word,
            );
            channel.say(&ctx.http, fail_string).await.unwrap();
            return;
        }

        let first_character = characters
            .first()
            .expect("Failed to get first character of word");

        // #TODO: Check that the word is playable
        if let Some(current_char) = self.get_current_character().await {
            let previous_word = self
                .get_previous_word()
                .await
                .expect("Couldn't get previous word");
            if &current_char != first_character {
                let bad_word_string = formatdoc! {r#"
                        {} Your word {} starts with {}. The previous word was {}, which ends in {}, so your word must start with {}
                "#, message.author.mention(), word, first_character, previous_word.word, current_char, current_char};

                channel.say(&ctx.http, bad_word_string).await.unwrap();
                return;
            }
        }
        // If all the characters are hiragana, play the word.

        let played_message = format!("{} Played word: {}", message.author.mention(), word);
        channel.say(&ctx.http, played_message).await.unwrap();
        self.words.lock().await.push(Word::new(
            message.author.clone(),
            word.to_string(),
            self.get_display_name(&ctx, &message).await,
        ));
    }

    async fn get_display_name(&self, ctx: &Context, message: &Message) -> String {
        message
            .member(&ctx.http)
            .await
            .expect("Could not find member")
            .display_name()
            .to_string()
    }

    async fn history(&self, ctx: Context, message: Message) {
        println!("Logging played words:");
        let mut log_message = String::from("Game history: \n");
        for word in self.words.lock().await.iter() {
            log_message.push_str(format!("{}: {}\n", word.display_name, word.word).as_str())
        }
        message
            .channel_id
            .say(&ctx.http, log_message)
            .await
            .unwrap();
    }

    async fn get_previous_word(&self) -> Option<Word> {
        self.words.lock().await.last().cloned()
    }

    async fn get_current_character(&self) -> Option<Hiragana> {
        match self.get_previous_word().await {
            None => None,
            Some(word) => Some(Hiragana::new(word.word.chars().last().unwrap()).unwrap()),
        }
    }

    async fn show_previous_word(&self, ctx: Context, message: Message) {
        let previous_word = self.get_previous_word().await;
        let mut _word_string: &str;
        let word_string = match previous_word {
            None => format!("No word has been played yet."),
            Some(previous_word) => {
                format!(
                    "The last word was {} by {}",
                    previous_word.word,
                    self.get_display_name(&ctx, &message).await,
                )
            }
        };

        message
            .channel_id
            .say(&ctx.http, word_string)
            .await
            .unwrap();
    }
}

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
            1 => self.help(ctx, message).await,
            2 => match split[1] {
                "help" => self.help(ctx, message).await,
                "history" => self.history(ctx, message).await,
                "word" => self.show_previous_word(ctx, message).await,
                _ => self.play(ctx, message, split[1]).await,
            },
            _ => self.not_recognised(ctx, message).await,
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
