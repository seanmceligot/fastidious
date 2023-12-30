use ansi_term::Colour;
use config::ConfigError;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::{ffi::OsString, fmt};
use thiserror::Error;

#[derive(Debug, Copy, Clone)]
pub enum Verb {
    Would,
    Live,
    Skipped,
}
impl Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{:?}", self)
        Debug::fmt(self, f)
    }
}
pub fn color_from_verb(verb: Verb) -> Colour {
    match verb {
        Verb::Would => Colour::Yellow,
        Verb::Live => Colour::Green,
        Verb::Skipped => Colour::Yellow,
    }
}
pub fn log_cmd_action(action: &'static str, verb: Verb, cli: String) {
    let color: Colour = color_from_verb(verb);
    println!(
        "{}: {}: {}",
        color.paint(verb.to_string()),
        color.paint(action),
        color.paint(cli),
    );
}
pub fn log_path_action(action: &'static str, verb: Verb, path: &Path) {
    let color: Colour = color_from_verb(verb);
    println!(
        "{}: {}: {}",
        color.paint(verb.to_string()),
        color.paint(action),
        color.paint(path.display().to_string()),
    );
}
