#[macro_export]
macro_rules! bot_commands {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                #[command(name = $cmd_name:literal, desc = $cmd_desc:literal $(, parser = $parser:expr)? $(, unit = $unit:ident)?)]
                $variant:ident $(($($t:ty),+))?
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $variant $(($($t),+))?
            ),+
        }

        impl $name {
            $vis fn bot_commands() -> Vec<frankenstein::types::BotCommand> {
                vec![
                    $(
                        frankenstein::types::BotCommand {
                            command: $cmd_name.to_string(),
                            description: $cmd_desc.to_string(),
                        }
                    ),+
                ]
            }

            $vis fn parse(command: &str, argument: &str, quote: &str) -> Result<(Self, Option<String>), String> {
                let command = command
                    .trim_start_matches("/")
                    .split('@')
                    .next()
                    .unwrap_or("");

                if command.is_empty() {
                    return Err("command is empty".to_string());
                }

                let quote_or_argument = if argument.is_empty() { quote } else { argument }.to_string();

                match command {
                    $(
                        $cmd_name => {
                            $crate::parse_variant!($variant, quote_or_argument, argument, quote $(, parser = $parser)? $(, unit = $unit)?)
                        }
                    )+
                    _ => Err("not implemented".to_string()),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! parse_variant {
    ($variant:ident, $qa:expr, $arg:expr, $quote:expr, parser = $parser:expr) => {
        $parser($qa, $arg, $quote)
    };
    ($variant:ident, $qa:expr, $arg:expr, $quote:expr, unit = true) => {
        Ok((Self::$variant, None))
    };
    ($variant:ident, $qa:expr, $arg:expr, $quote:expr) => {
        Ok((Self::$variant($qa), None))
    };
}
