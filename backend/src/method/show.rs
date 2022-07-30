use std::env;
use std::path::PathBuf;

use super::Error;
use crate::song::SongPool;

fn get_color_code(name: &str) -> &str {
    match name {
        "black" => "\x1b[30m",
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "matgenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        "Black" => "\x1b[30;1m",
        "Red" => "\x1b[31;1m",
        "Green" => "\x1b[32;1m",
        "Yellow" => "\x1b[33;1m",
        "Blue" => "\x1b[34;1m",
        "Magenta" => "\x1b[35;1m",
        "Cyan" => "\x1b[36;1m",
        "White" => "\x1b[37;1m",
        _ => "",
    }
}

enum Mode {
    OnlyDefault,
    OnlyTranslation,
    Both,
}

struct Config {
    path: PathBuf,
    new_key: String,
    text_color: Option<String>,
    chord_color: Option<String>,
    keyword_color: Option<String>,
    translation_color: Option<String>,
    spaces: String,
    mode: Mode,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Self, Error> {
        let mut filename: Option<String> = None;
        let mut new_key = String::from("Self");
        let mut text_color: Option<String> = None;
        let mut chord_color: Option<String> = None;
        let mut keyword_color: Option<String> = None;
        let mut translation_color: Option<String> = None;
        let mut spacecnt = 2;
        let mut mode = Mode::OnlyDefault;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-k" => match args.next() {
                    Some(key) => new_key = key,
                    None => {
                        return Err(Error::ParseArgs(String::from(
                            "No value for option -k given",
                        )))
                    }
                },
                "-Ct" => match args.next() {
                    Some(color) => text_color = Some(get_color_code(&color).to_string()),
                    None => {
                        return Err(Error::ParseArgs(String::from(
                            "No value for option -Ct given",
                        )))
                    }
                },
                "-Cc" => match args.next() {
                    Some(color) => chord_color = Some(get_color_code(&color).to_string()),
                    None => {
                        return Err(Error::ParseArgs(String::from(
                            "No value for option -Cc given",
                        )))
                    }
                },
                "-Ck" => match args.next() {
                    Some(color) => keyword_color = Some(get_color_code(&color).to_string()),
                    None => {
                        return Err(Error::ParseArgs(String::from(
                            "No value for option -Ck given",
                        )))
                    }
                },
                "-CT" => match args.next() {
                    Some(color) => translation_color = Some(get_color_code(&color).to_string()),
                    None => {
                        return Err(Error::ParseArgs(String::from(
                            "No value for option -CT given",
                        )))
                    }
                },
                "-s" => match args.next() {
                    Some(s) => spacecnt = s.parse::<usize>().unwrap_or(2),
                    None => {
                        return Err(Error::ParseArgs(String::from(
                            "No value for option -s given",
                        )))
                    }
                },
                "-T" => mode = Mode::OnlyTranslation,
                "-Ta" => mode = Mode::Both,
                f => filename = Some(String::from(f)),
            }
        }

        let path = match filename {
            Some(f) => PathBuf::from(&f),
            None => return Err(Error::ParseArgs(String::from("No filename given"))),
        };
        let mut spaces = String::new();
        for _ in 0..spacecnt {
            spaces.push_str(" ");
        }
        Ok(Config {
            path,
            new_key,
            text_color,
            chord_color,
            keyword_color,
            translation_color,
            spaces,
            mode,
        })
    }
}

pub fn show(args: env::Args) -> Result<(), Error> {
    let config = Config::new(args)?;
    let song = SongPool::lazy_load_file(config.path, &config.new_key)?;

    let (show_default, show_translation_text, show_translation_chord) = match &config.mode {
        Mode::Both => (true, true, false),
        Mode::OnlyDefault => (true, false, false),
        Mode::OnlyTranslation => (false, true, true),
    };
    let translation_text_color = if show_default {
        &config.translation_color
    } else {
        &config.text_color
    };

    for section in song.sections {
        if let Some(keyword) = section.keyword {
            match &config.keyword_color {
                Some(color) => println! {"{}{}\x1b[0m", color, keyword},
                None => println! {"{}", keyword},
            }
        }
        for line in section.lines {
            if show_default {
                if let Some(chord) = line.chord {
                    match &config.chord_color {
                        Some(color) => println! {"{}{}{}\x1b[0m", config.spaces, color, chord},
                        None => println! {"{}{}", config.spaces, chord},
                    }
                }
                if let Some(text) = line.text {
                    match &config.text_color {
                        Some(color) => println! {"{}{}{}\x1b[0m", config.spaces, color, text},
                        None => println! {"{}{}", config.spaces, text},
                    }
                }
            }
            if show_translation_chord {
                if let Some(translation_chord) = line.translation_chord {
                    match &config.chord_color {
                        Some(color) => {
                            println! {"{}{}{}\x1b[0m", config.spaces, color, translation_chord}
                        }
                        None => println! {"{}{}", config.spaces, translation_chord},
                    }
                }
            }
            if show_translation_text {
                if let Some(translation_text) = line.translation_text {
                    match translation_text_color {
                        Some(color) => {
                            println! {"{}{}{}\x1b[0m", config.spaces, color, translation_text}
                        }
                        None => println! {"{}{}", config.spaces, translation_text},
                    }
                }
            }
        }
        println!("");
    }
    Ok(())
}
