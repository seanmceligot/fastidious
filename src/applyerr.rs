#![allow(dead_code)]

use ansi_term::Colour;
use config::ConfigError;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{ffi::OsString, fmt};
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Warnings")]
    Warn,

    #[error("file read error: {0}")]
    FileReadError(String),

    #[error("path not found: {0}")]
    PathNotFound(String),

    #[error("path not found: {0}")]
    PathBufNotFound(PathBuf),

    #[error("file create error: {0}")]
    FileCreateError(String),

    #[error("file read error: {0}")]
    FileWriteError(String),

    #[error("Variable not found {0}")]
    VarNotFound(String),

    #[error("Name not found {0}")]
    NameNotFound(String),

    #[error("Terminated without status code: ")]
    CmdExitedPrematurely,

    #[error("Non zero exit status code {0} ")]
    NotZeroExit(i32),

    #[error("Io Error {0}")]
    IoError(#[from] std::io::Error),

    #[error("Command not found {0}")]
    CommandNotFound(String),

    #[error("Expected argument: {0}")]
    ExpectedArg(String),

    #[error("Expected argument: {0}")]
    UnExpectedArg(String),

    #[error("Insufficient Privileges {0}")]
    InsufficientPrivileges(String),

    #[error("#[from] Error in config")]
    ConfigError(ConfigError),

    // #[error("Path not found {0}")]
    // PathNotFound(String),
    #[error("Path not found")]
    PathNotFound0,

    #[error("Diff Error {0}")]
    DiffFailed(String),

    #[error("not a file {0}")]
    NotAFile(PathBuf),

    #[error("Copy Error {0} {1} {2}")]
    CopyError(PathBuf, PathBuf, String),

    #[error("Execute Error {0}")]
    ExecError(String),

    #[error("No Parent Dir {0}")]
    NoParent(String),

    #[error("Script Error {0}")]
    ScriptError(String),
}

#[derive(Debug, Copy, Clone)]
pub enum Verb {
    Would,
    Live,
    Skipped,
}
impl fmt::Display for Verb {
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
