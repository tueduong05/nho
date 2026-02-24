use std::time::Duration;

use bytes::Bytes;

pub enum Command {
    Ping,
    Get(Bytes),
    Set(Bytes, Bytes, Option<Duration>),
    Unknown,
}

impl Command {
    pub fn from_args(args: Vec<Bytes>) -> Self {
        if args.is_empty() {
            return Command::Unknown;
        }

        let command_name = String::from_utf8_lossy(&args[0]).to_lowercase();

        match command_name.as_str() {
            "ping" => Command::Ping,
            "get" if args.len() == 2 => Command::Get(args[1].clone()),
            "set" => {
                if args.len() == 3 {
                    return Command::Set(args[1].clone(), args[2].clone(), None);
                }

                if args.len() == 5 {
                    let key = args[1].clone();
                    let value = args[2].clone();
                    let option = String::from_utf8_lossy(&args[3]).to_lowercase();

                    let amount = String::from_utf8_lossy(&args[4]).parse::<u64>().ok();

                    match (option.as_str(), amount) {
                        ("ex", Some(sec)) => {
                            Command::Set(key, value, Some(Duration::from_secs(sec)))
                        }
                        ("px", Some(ms)) => {
                            Command::Set(key, value, Some(Duration::from_millis(ms)))
                        }
                        _ => Command::Unknown,
                    }
                } else {
                    Command::Unknown
                }
            }
            _ => Command::Unknown,
        }
    }
}
