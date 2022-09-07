use serenity::model::user::User;

#[derive(Debug, Clone)]
pub struct Word {
    pub author: User,
    pub word: String,
    pub display_name: String,
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.author.tag(), self.word)
    }
}

impl Word {
    pub fn new(user: User, word: String, display_name: String) -> Word {
        Word {
            author: user,
            word,
            display_name,
        }
    }
}
