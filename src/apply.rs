use ansi_term::Colour::{Green, Red, Yellow};
use cmd::execute_script;
use seahorse::{Flag, FlagType};
//use failure::Error;
use std::{collections::HashMap, env, io::{self, Write}, path::{Path, PathBuf}, process::Command};
use applyerr::ApplyError;

use crate::cmd;

pub(crate) fn execute_apply(_name: &str, script: &cmd::Script, vars: HashMap<String, String>) -> bool {
    match cmd::execute_script(script, vars) {
        Ok(_) => {
            println!("{}", Green.paint("Applied"));
            true
        }
        Err(_e) => {
            println!("{}", Yellow.paint("Apply Failed"));
            false
        }
    }
}
pub(crate) fn is_applied(_name: &str, script: &cmd::Script, vars: HashMap<String, String>) -> bool {
    match execute_script(script, vars) {
        Ok(_) => {
            println!("{}", Green.paint("Applied"));
            true
        }
        Err(_e) => {
            println!("{}", Yellow.paint("Unapplied"));
            false
        }
    }
}
