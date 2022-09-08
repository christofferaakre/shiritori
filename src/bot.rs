use indoc::formatdoc;
use kanji::Character;
use kanji::Hiragana;

use serenity::model::channel::Message;

use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::constants::HELP_STRING;
use crate::word::Word;

type Words = Mutex<Vec<Word>>;

#[derive(Debug)]
pub struct Bot {
    words: Words,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            words: Mutex::new(vec![]),
        }
    }

    async fn say(&self, ctx: &Context, channel: &ChannelId, message: &impl std::fmt::Display) {
        if let Err(why) = channel.say(&ctx.http, message).await {
            eprintln!("Error sending message: {:?}", why);
        }
    }

    pub async fn help(&self, ctx: Context, message: Message) {
        self.say(&ctx, &message.channel_id, &HELP_STRING).await;
    }

    pub async fn not_recognised(&self, ctx: Context, message: Message) {
        let error_message = format!("{} is not a recognised command", message.content.clone());
        println!("{}", error_message);
        self.say(&ctx, &message.channel_id, &error_message).await
    }

    async fn check_loss(&self, characters: &Vec<Hiragana>) -> bool {
        let last_character = characters
            .last()
            .expect("Failed to get last character of word");

        // #TODO: End game, update leaderbord, etc.
        if last_character == &Hiragana::new('ん').unwrap() {
            return true;
        }

        return false;
    }

    async fn is_word_playable(&self, characters: &Vec<Hiragana>) -> bool {
        let first_character = characters
            .first()
            .expect("Failed to get first character of word");

        // If there is a current char, the first char must be equal to that
        if let Some(current_char) = self.get_current_character().await {
            let previous_word = self
                .get_previous_word()
                .await
                .expect("Couldn't get previous word");

            return first_character.get() == current_char.get();
        }
        // Otherwise, any first character is fine
        else {
            true
        }
    }

    async fn play_word(&self, ctx: &Context, message: &Message, word: Word) {
        let played_message = format!("{} Played word: {}", message.author.mention(), word);
        self.say(&ctx, &message.channel_id, &played_message).await;

        self.words.lock().await.push(word);
    }

    pub async fn try_play_word(&self, ctx: Context, message: Message, word: &str) {
        let channel = message.channel_id;
        let characters = word.chars().map(Character::new);
        // Kind of an ugly way to do this, but it works
        let all_hiragana = characters.clone().all(|c| kanji::is_hiragana(c.get()));

        // check if message contains non-hiragana characters
        if !all_hiragana {
            println!("Word {} contained non-hiragana characters", word);
            self.not_recognised(ctx, message).await;
            return;
        }

        // convert Character structs to Hiragana structs
        let characters: Vec<Hiragana> = characters
            .map(|c| Hiragana::new(c.get()).unwrap())
            .collect();

        let word = Word::new(
            message.author.clone(),
            word.to_string(),
            self.get_display_name(&ctx, &message).await,
        );

        if self.check_loss(&characters).await {
            let fail_string = format!(
                "{} Your word {} ends in ん. Better luck next time!",
                message.author.mention(),
                word.word,
            );
            self.say(&ctx, &message.channel_id, &fail_string).await;
            return;
        }

        if !self.is_word_playable(&characters).await {
            let first_character = characters.first().unwrap();
            let current_char = message.content.chars().last().unwrap();
            let bad_word_string = formatdoc! {r#"{} Your word {} starts with {}. The previous word was {}, which ends in {}, so your word must start with {}"#, message.author.mention(), word, first_character, self.words.lock().await.last().unwrap().word, current_char, current_char};

            self.say(&ctx, &channel, &bad_word_string).await;
            return;
        }

        // If the word is playable, play it.
        self.play_word(&ctx, &message, word).await;
    }

    pub async fn get_display_name(&self, ctx: &Context, message: &Message) -> String {
        message
            .member(&ctx.http)
            .await
            .expect("Could not find member")
            .display_name()
            .to_string()
    }

    pub async fn history(&self, ctx: Context, message: Message) {
        let mut log_message = String::from("Game history: \n");
        for word in self.words.lock().await.iter() {
            log_message.push_str(format!("{}: {}\n", word.display_name, word.word).as_str())
        }
        self.say(&ctx, &message.channel_id, &log_message).await;
    }

    pub async fn get_previous_word(&self) -> Option<Word> {
        self.words.lock().await.last().cloned()
    }

    pub async fn get_current_character(&self) -> Option<Hiragana> {
        match self.get_previous_word().await {
            None => None,
            Some(word) => Some(Hiragana::new(word.word.chars().last().unwrap()).unwrap()),
        }
    }

    pub async fn show_previous_word(&self, ctx: Context, message: Message) {
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
        self.say(&ctx, &message.channel_id, &word_string).await;
    }
}
