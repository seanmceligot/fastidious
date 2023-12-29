#![allow(dead_code)]

use ansi_term::Colour;
use config::ConfigError;
use std::path::{Path, PathBuf};
use std::{ffi::OsString, fmt};
use thiserror::Error;

use std::fmt::Debug;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ErrorMessage {}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("Error: {0}")]
    Error(String),

    #[error("Warnings")]
    Warn,

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

    #[error("Command not found {0}")]
    CommandNotFound(String),

    #[error("Expected argument: {0}")]
    ExpectedArg(String),

    #[error("Expected argument: {0}")]
    UnExpectedArg(String),

    #[error("Insufficient Privileges {0}")]
    InsufficientPrivileges(String),

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

    #[error("Io Error {0}")]
    IoError(#[from] std::io::Error),

    #[error("#[from] Error in config")]
    ConfigError(ConfigError),

    #[error("Yaml Read Error {0}")]
    YamlReadError(#[from] serde_yaml::Error),

    #[error("AnyHow Error {0}")]
    AnyHowError(#[from] anyhow::Error),
}

impl From<ConfigError> for ApplyError {
    fn from(error: ConfigError) -> Self {
        ApplyError::ConfigError(error)
    }
}
