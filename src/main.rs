#![allow(unused_imports)]
#![feature(array_chunks)]

extern crate ansi_term;
extern crate config;
extern crate dirs;
extern crate env_logger;
extern crate getopts;
extern crate glob;
#[macro_use]
extern crate log;
extern crate clap;
extern crate regex;
extern crate seahorse;
extern crate serde_derive;
extern crate simple_logger;
extern crate thiserror;
extern crate which;

use crate::cmd::Vars;
use clap::{Parser, Subcommand};
use config::builder::{BuilderState, ConfigBuilder};
use config::Config;
use files::DestFile;
use files::Mode;

use ansi_term::Colour::{Green, Red, Yellow};
use seahorse::{Flag, FlagType};
//use failure::Error;
use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};
mod applyerr;
use applyerr::ApplyError;
mod apply;
mod cmd;
mod configfile;
mod diff;
mod dryrun;
mod files;
mod fs;
mod template;
mod userinput;

use apply::execute_apply;

use crate::cmd::VirtualFile;

#[test]
fn test_appply() -> Result<(), ApplyError> {
    let apply_script = VirtualFile::in_memory_shell(String::from("touch test1.tmp"));
    let is_applied = VirtualFile::in_memory_shell(String::from("test -f test1.tmp"));

    let name_config: HashMap<String, String> = HashMap::new();
    do_is_applied(name_config.clone(), &is_applied)?;
    do_apply(name_config, &apply_script, files::Mode::Active)?;
    Ok(())
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "fastidious")]
#[command(about = "fastidious", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes

#[derive(Debug, Subcommand)]
enum Commands {
    /// Clones repos
    #[command(arg_required_else_help = true)]
    Dryrun {
        #[clap(short, long)]
        active: bool,
        #[clap(short, long)]
        passive: bool,
        #[clap(short, long)]
        interactive: bool,
        #[arg(short, long,num_args=0..)]
        var: Vec<String>,
        #[arg(last = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    Template {
        #[clap(short, long)]
        active: bool,
        #[clap(short, long)]
        passive: bool,
        #[clap(short, long)]
        interactive: bool,
        #[arg(short, long,num_args=0..)]
        var: Vec<String>,
        #[arg(short = 'I', long)]
        infile: Option<PathBuf>,
        #[arg(short, long)]
        out: PathBuf,
        #[arg(last = true, allow_hyphen_values = true)]
        data: Option<Vec<String>>,
    },
    Apply {
        #[arg(short, long)]
        name: Option<String>,
        #[clap(short, long)]
        active: bool,
        #[clap(short, long)]
        passive: bool,
        #[clap(short, long)]
        interactive: bool,
        #[arg(short = 'I', long)]
        ifnot: Option<String>,
        #[arg(short, long)]
        then: String,
        #[arg(short, long, num_args=0..)]
        var: Vec<String>,
    },
    IsApplied {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        ifnot: String,
    },
}

fn main1() -> Result<(), ApplyError> {
    env_logger::init();
    debug!("debug enabled");
    info!("info enabled");

    info!("info enabled");
    let _conf = Config::builder()
        .add_source(config::Environment::with_prefix("FASTIDIOUS"))
        .add_source(config::File::with_name("fastidious").required(false))
        .build()
        .map_err(|e| ApplyError::ConfigError(e))?;
    info!("after conf");

    let args = Cli::parse();

    match args.command {
        Commands::Dryrun {
            active,
            passive,
            interactive,
            var,
            cmd,
        } => {
            let mode = get_mode(active, passive, interactive);
            let vars = crate::cmd::to_vars_split_odd(var);
            debug!("vars {:#?}", vars);
            debug!("cmd {:#?}", cmd);
            dryrun::dryrun(mode, vars, cmd)?
        }
        Commands::Apply {
            name,
            active,
            interactive,
            ifnot,
            then,
            var,
            passive,
        } => {
            let mode = get_mode(active, passive, interactive);
            let vars = crate::cmd::to_vars_split_odd(var);
            apply_action(name, mode, ifnot, then, vars)?
        }
        Commands::IsApplied { name, ifnot } => {
            debug!("maybe_ifnot {:?}", ifnot);
            debug!("name{:?}", name);
            todo!()
        }
        Commands::Template {
            active,
            interactive,
            passive,
            var,
            data,
            infile,
            out,
        } => {
            let mode = get_mode(active, interactive, passive);
            let vars = crate::cmd::to_vars_split_odd(var);
            let output_file = DestFile::new(out);
            let str_data = match data {
                Some(v) => Some(v.join(" ")),
                None => None,
            };
            debug!("str_data {:?}", str_data);
            dryrun::do_template(mode, vars, str_data, infile, output_file)?
        }
    }
    Ok(())
}

fn get_mode(active: bool, _passive: bool, interactive: bool) -> files::Mode {
    if active {
        files::Mode::Active
    } else if interactive {
        files::Mode::Interactive
    } else {
        files::Mode::Passive
    }
}
fn apply_action(
    maybe_name: Option<String>,
    mode: Mode,
    ifnot: Option<String>,
    _then: String,
    _var: Vars,
) -> Result<(), ApplyError> {
    debug!("apply_action");
    let maybe_name_str = maybe_name.as_deref();

    let conf = Config::builder()
        .add_source(config::Environment::with_prefix("FASTIDIOUS"))
        .add_source(config::File::with_name("fastidious").required(false))
        .build()
        .map_err(|e| ApplyError::ConfigError(e))?;

    let name_config: HashMap<String, String> = if let Some(name) = maybe_name_str {
        configfile::scriptlet_config(&conf, name).expect("scriptlet_config")
    } else {
        HashMap::new()
    };
    let maybe_is_applied_script =
        lookup_is_applied_script(maybe_name_str, &name_config, &conf, ifnot.as_deref());
    let is_applied_script = maybe_is_applied_script?;

    let is = do_is_applied(name_config.clone(), &is_applied_script).map_err(|e| {
        ApplyError::ScriptError(format!("script error {:?} {:?}", is_applied_script, e))
    })?;

    if is {
        debug!("already applied");
        Ok(())
    } else {
        let maybe_apply_script =
            lookup_apply_script(maybe_name.as_deref(), &name_config, &conf, ifnot.as_deref());
        let apply_script = maybe_apply_script.unwrap();

        do_apply(name_config, &apply_script, mode)
    }
}

fn lookup_is_applied_script(
    maybe_name: Option<&str>,
    name_config: &HashMap<String, String>,
    conf: &config::Config,
    maybe_ifnot: Option<&str>,
) -> Result<VirtualFile, ApplyError> {
    let script_arg_name = "ifnot";
    let script_param_name = "is_applied";
    let script_file_name = "is-applied";

    lookup_script(
        script_arg_name,
        maybe_name,
        name_config,
        script_param_name,
        conf,
        script_file_name,
        maybe_ifnot,
    )
}
fn lookup_apply_script(
    maybe_name: Option<&str>,
    name_config: &HashMap<String, String>,
    conf: &config::Config,
    maybe_ifnot: Option<&str>,
) -> Result<VirtualFile, ApplyError> {
    let script_arg_name = "then";
    let script_param_name = "apply";
    let script_file_name = "apply";

    lookup_script(
        script_arg_name,
        maybe_name,
        name_config,
        script_param_name,
        conf,
        script_file_name,
        maybe_ifnot,
    )
}
fn lookup_script(
    _script_arg_name: &str,
    maybe_name: Option<&str>,
    name_config: &HashMap<String, String>,
    script_param_name: &str,
    conf: &config::Config,
    script_file_name: &str,
    maybe_ifnot: Option<&str>,
) -> Result<VirtualFile, ApplyError> {
    debug!("maybe_ifnot {:?}", maybe_ifnot);
    let _script = name_config
        .get(script_param_name)
        .map(|source| cmd::VirtualFile::in_memory_shell(source.to_string()));
    if let Some(name) = maybe_name {
        let slet = configfile::find_scriptlet(conf, name, script_file_name);
        debug!("scriptlet {:?}", slet);
    }
    todo!();
}
fn _try_is_applied_action(
    maybe_name: Option<&str>,
    maybe_ifnot: Option<&str>,
) -> Result<bool, ApplyError> {
    println!("is_applied_action");
    let conf = Config::builder()
        .add_source(config::Environment::with_prefix("FASTIDIOUS"))
        .add_source(config::File::with_name("fastidious").required(false))
        .build()
        .map_err(|e| ApplyError::ConfigError(e))?;

    let name_config: HashMap<String, String> = if let Some(name) = maybe_name {
        configfile::scriptlet_config(&conf, name).expect("scriptlet_config")
    } else {
        HashMap::new()
    };
    let maybe_is_applied_script =
        lookup_is_applied_script(maybe_name.as_deref(), &name_config, &conf, maybe_ifnot);
    let is_applied_script = maybe_is_applied_script?;

    do_is_applied(name_config, &is_applied_script)
}

fn do_apply(
    name_config: HashMap<String, String>,
    script_path: &cmd::VirtualFile,
    mode: files::Mode,
) -> Result<(), ApplyError> {
    debug!("do_apply params {:#?} {:?}", name_config, mode);
    execute_apply(script_path, name_config, mode);
    Ok(())
}

fn do_is_applied(
    name_config: HashMap<String, String>,
    script: &cmd::VirtualFile,
) -> Result<bool, ApplyError> {
    debug!("do_is_applied params {:#?}", name_config);
    debug!("do_is_applied script {:?}", script);
    Ok(apply::is_applied(script, name_config))
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
