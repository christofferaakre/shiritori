use indoc::indoc;

pub const DISCORD_BOT_TOKEN: &str = include_str!("../DISCORD_BOT_TOKEN");

pub const PREFIX: &str = "~";

pub const HELP_STRING: &str = indoc! {r#"
    Play shiritori!
    Commands:
        !shiri / !shiri help - Display this help message
"#};
