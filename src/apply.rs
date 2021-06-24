use ansi_term::Colour::{Green, Red, Yellow};
use seahorse::{Flag, FlagType};
//use failure::Error;
use applyerr::ApplyError;
use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    cmd::{self, Args, Vars},
    dryrun::{self, execute},
};

pub(crate) fn execute_apply(script: &cmd::VirtualFile, vars: Vars) -> bool {
    let args = Args::new();
    match dryrun::execute(crate::files::Mode::Active, script, args, &vars) {
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
pub(crate) fn is_applied(script: &cmd::VirtualFile, vars: HashMap<String, String>) -> bool {
    let args = Args::new();
    match execute(crate::files::Mode::Active, script, args, &vars) {
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
