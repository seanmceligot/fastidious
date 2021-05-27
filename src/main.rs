#![allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate dotenv;
//extern crate env_logger;
extern crate seahorse;
//#[macro_use]
extern crate config;
extern crate dirs;
extern crate serde_derive;
//extern crate toml;
extern crate tempfile;

use ansi_term::Colour::{Green, Red, Yellow};
//use failure::Error;
use std::{collections::HashMap, env, io::{self, Write}, path::{Path, PathBuf}, process::Command};
mod applyerr;
use applyerr::ApplyError;
///mod action;
mod configfile;

#[test]
fn test_appply() -> Result<(), ApplyError> {
    let apply_script = Script::InMemory(String::from("touch test1.tmp"));
    let is_applied = Script::InMemory(String::from("test -f test1.tmp"));
    let name = "example1";   

    let name_config : HashMap<String,String>  = HashMap::new(); 
    do_is_applied(name_config.clone(), &is_applied, name)?; 
    do_apply(name_config,&apply_script, name)?;    
    Ok(())    
}

enum Script {
    FsPath(PathBuf),
    InMemory(String)
}

fn main1() -> Result<(), ApplyError> {
    dotenv::dotenv().ok();
    env_logger::init();
    let args: Vec<String> = env::args().collect();

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
    .usage("applied action name")
    .action(apply_action)
    .command(apply_command)
    .command(is_applied_command);

    app.run(args);
    
  
    Ok(())
}
fn apply_action(c: &seahorse::Context) {
    println!("apply_action");
    let name: &str = c.args.first().unwrap();
    debug!("apply_action {}", name);

    let c1 = &mut config::Config::default();
    let conf = configfile::load_config(c1).unwrap();

    let name_config: HashMap<String, String> = configfile::scriptlet_config(conf, name).expect("scriptlet_config");
    let is_applied_script = Script::FsPath(configfile::find_scriptlet(conf, name, "is-applied"));
    do_is_applied(name_config.clone(), &is_applied_script, name).unwrap();    
    let apply_script = Script::FsPath(configfile::find_scriptlet(conf, name, "apply"));
    do_apply(name_config, &apply_script, name).unwrap();    

}
fn is_applied_action(c: &seahorse::Context) {
    println!("is_applied_action");
    let name: &str = c.args.first().unwrap();
    debug!("is_applied_action {}", name);
    
    let c1 = &mut config::Config::default();
    let conf = configfile::load_config(c1).unwrap();
    let name_config: HashMap<String, String> = configfile::scriptlet_config(conf, name).expect("scriptlet_config");

    let is_applied_script = Script::FsPath(configfile::find_scriptlet(conf, name, "is-applied"));

    do_is_applied(name_config, &is_applied_script, name).unwrap();
}

fn do_apply(name_config: HashMap<String,String>, script_path: &Script, name: &str) -> Result<(), ApplyError> {
    
    debug!("params {:#?}", name_config);
    execute_apply(name, script_path, name_config);
    Ok(())
}

fn do_is_applied(name_config: HashMap<String,String>, script_path: &Script, name: &str) -> Result<(), ApplyError> {
    debug!("params {:#?}", name_config);
    is_applied(name, &script_path, name_config);
    Ok(())
}

fn execute_script_file(cmdpath: &Path,  vars: HashMap<String, String>) -> Result<(), ApplyError> {
    let cmdstr = cmdpath.as_os_str();
    debug!("run: {:#?}", cmdstr);
    let output = Command::new("bash")
        .arg(cmdstr)
        .envs(vars)
        .output()
        .expect("cmd failed");
    io::stdout()
        .write_all(&output.stdout)
        .expect("error writing to stdout");
    match output.status.code() {
        Some(n) => {
            if n == 0 {
                println!(
                    "{} {}",
                    Green.paint("status code: "),
                    Green.paint(n.to_string())
                );
                Ok(())
            } else {
                println!(
                    "{} {}",
                    Red.paint("status code: "),
                    Red.paint(n.to_string())
                );
                Err(ApplyError::NotZeroExit(n))
            }
        }
        None => Err(ApplyError::CmdExitedPrematurely),
    }

}
fn execute_script(script: &Script,  vars: HashMap<String, String>) -> Result<(), ApplyError> {
    match script {
        Script::FsPath(path) => execute_script_file(path,vars),
        Script::InMemory(source) => {
            let mut t = tempfile::NamedTempFile::new().unwrap();
            t.write(source.as_bytes()).unwrap();
            debug!("execute {:?}", t.path());
            let r = execute_script_file(t.path(), vars);
            t.close().unwrap();
            r
        }
    }
}
fn execute_apply(_name: &str, script: &Script, vars: HashMap<String, String>) -> bool {
    match execute_script(script, vars) {
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
fn is_applied(_name: &str, script: &Script, vars: HashMap<String, String>) -> bool {
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
