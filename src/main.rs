#![allow(unused_imports)]
extern crate ansi_term;
extern crate config;
extern crate dirs;
extern crate dotenv;
extern crate env_logger;
extern crate getopts;
extern crate glob;
#[macro_use]
extern crate log;
extern crate regex;
extern crate seahorse;
extern crate serde_derive;
extern crate simple_logger;
extern crate tempfile;
extern crate thiserror;
//extern crate toml;
extern crate which;

use ansi_term::Colour::{Green, Red, Yellow};
use cmd::execute_script;
use seahorse::{Flag, FlagType};
//use failure::Error;
use std::{collections::HashMap, env, io::{self, Write}, path::{Path, PathBuf}, process::Command};
mod applyerr;
use applyerr::ApplyError;
///mod action;
mod configfile;
mod dryrun;
mod dryrunerr;
mod files;
mod filter;
mod fs;
mod cmd;
mod template;
mod userinput;
mod diff;
mod apply;

use apply::execute_apply;

#[test]
fn test_appply() -> Result<(), ApplyError> {
    let apply_script = cmd::Script::InMemory(String::from("touch test1.tmp"));
    let is_applied = cmd::Script::InMemory(String::from("test -f test1.tmp"));
    let name = "example1";   

    let name_config : HashMap<String,String>  = HashMap::new(); 
    do_is_applied(name_config.clone(), &is_applied, name)?; 
    do_apply(name_config,&apply_script, name)?;    
    Ok(())    
}

fn main1() -> Result<(), ApplyError> {
    dotenv::dotenv().ok();
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let dry_command = seahorse::Command::new("dry")
    .description("dry [name] if not already applied")
    .alias("d")
    .usage("dry(d) [name...]")
    .action(dry_action)
    .flag(Flag::new("active", FlagType::Bool).alias("A"))
    .flag(Flag::new("interactive", FlagType::Bool).alias("I"))
    ;

    let apply_command = seahorse::Command::new("apply")
    .description("apply [name] if not already applied")
    .alias("a")
    .usage("apply(a) [name...]")
    .action(apply_action);

    let is_applied_command = seahorse::Command::new("is_applied")
    .description("is_applied [name] if not already applied")
    .alias("i")
    .usage("is_applied(i) [name...]")
    .action(is_applied_action);
  
    let app = seahorse::App::new(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .version(env!("CARGO_PKG_VERSION"))
    .command(apply_command)
    .command(is_applied_command)
    .command(dry_command)
    ;
 
    app.run(args);
    
  
    Ok(())
}
fn dry_action(c: &seahorse::Context) {
    debug!("dry_action");
    let name: &str = c.args.first().unwrap();
    debug!("dry_action {}", name);

    let mut mode = files::Mode::Passive;
    if c.bool_flag("active") {
        mode = files::Mode::Active;
    }
    if c.bool_flag("interactive") {
        mode = files::Mode::Interactive;
    }
    
    dryrun::dryrun(c.args.iter(), mode);
}
fn apply_action(c: &seahorse::Context) {
    println!("apply_action");
    let name: &str = c.args.first().unwrap();
    debug!("apply_action {}", name);

    let c1 = &mut config::Config::default();
    let conf = configfile::load_config(c1).unwrap();

    let name_config: HashMap<String, String> = configfile::scriptlet_config(conf, name).expect("scriptlet_config");
    let is_applied_script = cmd::Script::FsPath(configfile::find_scriptlet(conf, name, "is-applied"));
    do_is_applied(name_config.clone(), &is_applied_script, name).unwrap();    
    let apply_script = cmd::Script::FsPath(configfile::find_scriptlet(conf, name, "apply"));
    do_apply(name_config, &apply_script, name).unwrap();    

}
fn is_applied_action(c: &seahorse::Context) {
    println!("is_applied_action");
    let name: &str = c.args.first().unwrap();
    debug!("is_applied_action {}", name);
    
    let c1 = &mut config::Config::default();
    let conf = configfile::load_config(c1).unwrap();
    let name_config: HashMap<String, String> = configfile::scriptlet_config(conf, name).expect("scriptlet_config");

    let is_applied_script = cmd::Script::FsPath(configfile::find_scriptlet(conf, name, "is-applied"));

    do_is_applied(name_config, &is_applied_script, name).unwrap();
}

fn do_apply(name_config: HashMap<String,String>, script_path: &cmd::Script, name: &str) -> Result<(), ApplyError> {
    
    debug!("params {:#?}", name_config);
    execute_apply(name, script_path, name_config);
    Ok(())
}

fn do_is_applied(name_config: HashMap<String,String>, script_path: &cmd::Script, name: &str) -> Result<(), ApplyError> {
    debug!("params {:#?}", name_config);
    apply::is_applied(name, &script_path, name_config);
    Ok(())
}


fn main() {
    let code = match main1() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            -1
        }
    };
    std::process::exit(code);
}
