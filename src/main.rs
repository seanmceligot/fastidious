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
extern crate thiserror;
//extern crate toml;
extern crate which;

use ansi_term::Colour::{Green, Red, Yellow};
use seahorse::{Flag, FlagType};
//use failure::Error;
use std::{collections::HashMap, env, io::{self, Write}, path::{Path, PathBuf}, process::Command};
mod applyerr;
use applyerr::ApplyError;
///mod action;
mod configfile;
mod dryrun;
mod files;
mod fs;
mod cmd;
mod template;
mod userinput;
mod diff;
mod apply;

use apply::execute_apply;

use crate::cmd::Script;

#[test]
fn test_appply() -> Result<(), ApplyError> {
    let apply_script = cmd::Script::InMemory(String::from("touch test1.tmp"));
    let is_applied = cmd::Script::InMemory(String::from("test -f test1.tmp")); 

    let name_config : HashMap<String,String>  = HashMap::new(); 
    do_is_applied(name_config.clone(), &is_applied)?; 
    do_apply(name_config,&apply_script)?;    
    Ok(())    
}

fn main1() -> Result<(), ApplyError> {
    dotenv::dotenv().ok();
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let dry_command = seahorse::Command::new("dry")
    .description("dryrun [name]")
    .alias("d")
    .action(dry_action)
    .flag(Flag::new("active", FlagType::Bool).alias("A"))
    .flag(Flag::new("interactive", FlagType::Bool).alias("I"))
    ;

    let apply_command = seahorse::Command::new("apply")
    .description("apply [name] if is_applied")
    .alias("a")
    .action(apply_action)
    .flag(Flag::new("active", FlagType::Bool).alias("A"))
    .flag(Flag::new("interactive", FlagType::Bool).alias("I"))
    .flag(Flag::new("iscript", FlagType::String).alias("P"))
    .flag(Flag::new("ascript", FlagType::String).alias("Z"))
    .flag(Flag::new("var", FlagType::String).alias("V"))
;

    // use apply::execute_apply;
    let is_applied_command = seahorse::Command::new("is_applied")
    .description("is_applied [name] if not already applied")
    .alias("i")
    .action(is_applied_action)
    .flag(Flag::new("iscript", FlagType::String).alias("P"))
    ;
  
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
    if c.args.is_empty() {
        dryrun::print_usage("noname dry COMMAND");
        return;
    } 
    if let Some(name) = c.args.first() {
        debug!("dry_action {}", name);
    }

    let mut mode = files::Mode::Passive;
    if c.bool_flag("active") {
        mode = files::Mode::Active;
    }
    if c.bool_flag("interactive") {
        mode = files::Mode::Interactive;
    }
    
    match dryrun::dryrun(c.args.iter(), mode) {
        Ok(_) => {},
        Err(e) => {
            println!(
                "{} {}",
                Red.paint("error:"),
                Red.paint(e.to_string())
            );
        }
    }    
 }
fn apply_action(c: &seahorse::Context) {
    match try_apply_action(c) {
        Ok(_) => (),
        Err(e) => error!("{:?}", e)
    }
}
fn is_applied_action(c: &seahorse::Context) {
    match try_is_applied_action(c) {
        Ok(_) => (),
        Err(e) => error!("{:?}", e)
    }
}
fn try_apply_action(c: &seahorse::Context) -> Result<(), ApplyError>{
    debug!("apply_action");
    let maybe_name = c.args.first();

    let c1 = &mut config::Config::default();
    let conf = configfile::load_config(c1).map_err(|e| ApplyError::ConfigError(e.to_string()))?;

    
    let name_config: HashMap<String, String> = if let Some(name) = maybe_name {
        configfile::scriptlet_config(conf, name).expect("scriptlet_config")
    } else {
        HashMap::new()
    };
    let maybe_is_applied_script = lookup_is_applied_script(c, maybe_name, &name_config, conf);    let is_applied_script = maybe_is_applied_script?;
    
    do_is_applied(name_config.clone(), &is_applied_script).unwrap();    


    let maybe_apply_script = lookup_apply_script(c, maybe_name, &name_config, conf);
    let apply_script = maybe_apply_script.unwrap();

    do_apply(name_config, &apply_script)
}

fn lookup_is_applied_script(c: &seahorse::Context, maybe_name: Option<&String>, name_config: &HashMap<String, String>, conf: &mut config::Config) -> Result<Script, ApplyError> {
    let script_arg_name = "iscript";
    let script_param_name = "is_applied";
    let script_file_name = "is-applied";

    lookup_script(c, script_arg_name, maybe_name, name_config, script_param_name, conf, script_file_name)
}
fn lookup_apply_script(c: &seahorse::Context, maybe_name: Option<&String>, name_config: &HashMap<String, String>, conf: &mut config::Config) -> Result<Script, ApplyError> {
    let script_arg_name = "ascript";
    let script_param_name = "apply";
    let script_file_name = "apply";
 
    lookup_script(c, script_arg_name, maybe_name, name_config, script_param_name, conf, script_file_name)
}
fn lookup_script(c: &seahorse::Context, script_arg_name: &str, maybe_name: Option<&String>, name_config: &HashMap<String, String>, script_param_name: &str, conf: &mut config::Config, script_file_name: &str) -> Result<Script, ApplyError> {
    let maybe_iscript = c.string_flag(script_arg_name);
    debug!("maybe_iscript {:?}", maybe_iscript);
    let maybe_is_applied_script = match maybe_iscript {
        Ok(s) => Ok(Script::InMemory(s)),
        Err(_e) => 
            match maybe_name {
                Some(name) => {
                    // check name_config for "is_applied"
                    match name_config.get(script_param_name) {
                        Some(source) => Ok(cmd::Script::InMemory(source.clone())),
                        None => Ok(cmd::Script::FsPath(configfile::find_scriptlet(conf, name, script_file_name)))
                    }
                },
                None => 
                    Err(ApplyError::VarNotFound(String::from(
                        format!("arg --{} or config {} or file {}", script_arg_name, script_param_name, script_file_name))))
            }
    };
    maybe_is_applied_script
}
fn try_is_applied_action(c: &seahorse::Context) -> Result<(), ApplyError> {
    println!("is_applied_action");
    let maybe_name = c.args.first();
    let c1 = &mut config::Config::default();
    let conf = configfile::load_config(c1).map_err(|e| ApplyError::ConfigError(e.to_string()))?;

    
    let name_config: HashMap<String, String> = if let Some(name) = maybe_name {
        configfile::scriptlet_config(conf, name).expect("scriptlet_config")
    } else {
        HashMap::new()
    };
    let maybe_is_applied_script = lookup_is_applied_script(c, maybe_name, &name_config, conf);
    let is_applied_script = maybe_is_applied_script?;
    
    do_is_applied(name_config, &is_applied_script)
 }

fn do_apply(name_config: HashMap<String,String>, script_path: &cmd::Script) -> Result<(), ApplyError> {
    
    debug!("do_apply params {:#?}", name_config);
    execute_apply(script_path, name_config);
    Ok(())
}

fn do_is_applied(name_config: HashMap<String,String>, script: &cmd::Script) -> Result<(), ApplyError> {
    debug!("do_is_applied params {:#?}", name_config);
    debug!("do_is_applied script {:?}", script);
    apply::is_applied(&script, name_config);
    // TODO: return Delta >= 0
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
