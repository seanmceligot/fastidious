#![allow(unused_imports)]

extern crate ansi_term;
extern crate config;
extern crate dirs;
extern crate env_logger;
extern crate getopts;
extern crate glob;
#[macro_use]
extern crate log;
extern crate anyhow;
extern crate clap;
extern crate regex;
extern crate seahorse;
extern crate serde_derive;
extern crate simple_logger;
extern crate thiserror;
extern crate which;

use crate::cmd::Vars;
use anyhow::Error;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::builder::{BuilderState, ConfigBuilder};
use config::Config;
use dryrun::ActionResult;
use files::DestFile;
use files::Mode;

use ansi_term::Colour::{Green, Red, Yellow};
use seahorse::{Flag, FlagType};

use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};
mod applyerr;
pub mod passive;
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
        out: Option<PathBuf>,
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
    // Save: save key and value to yaml file filename
    // example: fastidious save --key "key" --value "value" --filename "filename"
    Save {
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        value: String,
        #[arg(short, long)]
        filename: String,
    },
}

fn main1() -> Result<ActionResult, ApplyError> {
    env_logger::init();
    debug!("debug enabled");
    info!("info enabled");

    info!("info enabled");
    let _conf = Config::builder()
        .add_source(config::Environment::with_prefix("FASTIDIOUS"))
        .add_source(config::File::with_name("fastidious").required(false))
        .build()
        .map_err(ApplyError::ConfigError)?;
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
            dryrun::dryrun(mode, vars, cmd)
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
            apply_action(mode, ifnot, then, vars)
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
            let str_data = data.map(|v| v.join(" "));
            debug!("str_data {:?}", str_data);
            let output_file = match out {
                Some(of) => DestFile::new(of),
                None => DestFile::new(PathBuf::from("/dev/stdout")),
            };
            dryrun::do_template(mode, vars, str_data, infile, output_file).map(ActionResult::from)
        }
        Commands::Save {
            key,
            value,
            filename,
        } => save_key_val(key, value, filename),
    }
}
fn load_yaml_to_hashmap(filename: &str) -> Result<HashMap<String, String>, ApplyError> {
    let contents = std::fs::read_to_string(filename).map_err(ApplyError::IoError)?;

    let yaml: HashMap<String, String> =
        serde_yaml::from_str(&contents).map_err(ApplyError::YamlReadError)?;
    Ok(yaml)
}
fn save_key_val(key: String, value: String, filename: String) -> Result<ActionResult, ApplyError> {
    let filepath = Path::new(&filename);
    if filepath.exists() {
        let mut yaml = load_yaml_to_hashmap(&filename)?;
        yaml.insert(key, value);
        let mut file = std::fs::File::create(filepath)
            .with_context(|| format!("Error creating {}", filename))?;
        let yaml =
            serde_yaml::to_string(&yaml).with_context(|| format!("error parsing {}", filename))?;
        file.write_all(yaml.as_bytes())
            .with_context(|| format!("error writing {}", filename))?;
    } else {
        let mut file = std::fs::File::create(filepath)?;
        let mut map = HashMap::new();
        map.insert(key.clone(), value);
        let yaml = serde_yaml::to_string(&map)
            .with_context(|| format!("error adding yaml key {}", key))?;
        file.write_all(yaml.as_bytes())
            .with_context(|| format!("error writing yaml {}", filename))?;
    }
    Ok(ActionResult::Applied)
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
    mode: Mode,
    maybe_ifnot: Option<String>,
    then: String,
    vars: Vars,
) -> Result<ActionResult, ApplyError> {
    debug!("apply_action");

    let _conf = Config::builder()
        .add_source(config::Environment::with_prefix("FASTIDIOUS"))
        .add_source(config::File::with_name("fastidious").required(false))
        .build()?;

    let is = if let Some(ifnot) = maybe_ifnot {
        let is_applied_script = VirtualFile::InMemory(ifnot);
        do_is_applied(vars.clone(), &is_applied_script).map_err(|e| {
            ApplyError::ScriptError(format!("script error {:?} {:?}", is_applied_script, e))
        })
    } else {
        Ok(false)
    }?;

    if is {
        info!("Already applied");
        Ok(ActionResult::AlreadyApplied)
    } else {
        let apply_script = VirtualFile::InMemory(then);

        do_apply(vars, &apply_script, mode)
    }
}

fn lookup_script(
    maybe_name: Option<&str>,
    conf: &config::Config,
) -> Result<VirtualFile, ApplyError> {
    debug!("maybe_name {:?}", maybe_name);
    debug!("conf {:?}", conf);
    if let Some(name) = maybe_name {
        debug!("name {:?}", name);
        let slet = configfile::find_scriptlet(conf, name, name);
        debug!("scriptlet {:?}", slet);
        if slet.exists() {
            Ok(VirtualFile::FsPath(slet))
        } else {
            Err(ApplyError::PathBufNotFound(slet))
        }
    } else {
        Err(ApplyError::ScriptError(String::from("Not provideded")))
    }
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
        .map_err(ApplyError::ConfigError)?;

    let name_config: HashMap<String, String> = if let Some(name) = maybe_name {
        configfile::scriptlet_config(&conf, name).expect("scriptlet_config")
    } else {
        HashMap::new()
    };
    let maybe_is_applied_script = lookup_script(maybe_name, &conf);
    let is_applied_script = maybe_is_applied_script?;

    do_is_applied(name_config, &is_applied_script)
}

fn do_apply(
    name_config: HashMap<String, String>,
    script_path: &cmd::VirtualFile,
    mode: files::Mode,
) -> Result<ActionResult, ApplyError> {
    debug!("do_apply params {:#?} {:?}", name_config, mode);
    execute_apply(script_path, name_config, mode)
}

fn do_is_applied(
    vars: HashMap<String, String>,
    script: &cmd::VirtualFile,
) -> Result<bool, ApplyError> {
    debug!("do_is_applied script {:?}", script);
    debug!("do_is_applied params {:#?}", vars);
    Ok(apply::is_applied(script, vars))
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
