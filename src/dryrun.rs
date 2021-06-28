use ansi_term::Colour::{Green, Red, Yellow};
use applyerr::log_cmd_action;
use applyerr::ApplyError;
use applyerr::Verb;
use cmd::exectable_full_path;
use diff::create_or_diff;
use diff::diff;
use diff::update_from_template;
use diff::DiffStatus;
use env_logger::Env;
use files::DestFile;
use files::GenFile;
use files::SrcFile;
use getopts::Options;
use log::trace;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::slice::Iter;
use std::str;
use template::{generate_recommended_file, replace_line, ChangeString};
use userinput::ask;
use crate::cmd::Args;
use crate::cmd::VirtualFile;
use crate::cmd::Vars;
use crate::files::Mode;

pub(crate) fn print_usage(program: &str) {
    println!("{}", program);
    println!("v key value            set template variable ");
    println!("t infile outfile       copy infile to outfile replacing @@key@@ with value  ");
    println!("x command arg1 arg2    run command  ");
    println!("-- x command -arg      run command (add -- to make sure hyphens are passed on");
}
#[derive(Debug)]
pub enum Action {
    Template(VirtualFile, String),
    Execute(VirtualFile, Args),
    Error(ApplyError),
    None,
}
#[derive(Debug)]
enum Type {
    Template,
    Execute,
    //InputFile,
    //OutputFile,
    Variable,
    Unknown,
}
#[test]
fn test_parse_type() {
    match parse_type(&String::from("t")) {
        Type::Template => {}
        _ => panic!("expected Template"),
    }
    match parse_type(&String::from("x")) {
        Type::Execute => {}
        _ => panic!("expected Execute"),
    }
    match parse_type(&String::from("v")) {
        Type::Variable => {}
        _ => panic!("expected Template"),
    }
}
fn parse_type(input: &str) -> Type {
    match input {
        "t" => Type::Template,
        "x" => Type::Execute,
        "v" => Type::Variable,
        _ => {
            debug!("Unknown {}", input);
            Type::Unknown
        }
    }
}
fn process_template_file<'t>(
    mode: Mode,
    vars: Vars,
    template: &SrcFile,
    dest: &DestFile,
) -> Result<DiffStatus, ApplyError> {
    let gen = generate_recommended_file(vars, template)?;
    create_or_diff(mode, template, dest, &gen)
}

#[test]
fn test_execute_active() -> Result<(), ApplyError> {
    let always_true = VirtualFile::FsPath(PathBuf::from("/bin/true"));
    let always_false = VirtualFile::FsPath(PathBuf::from("/bin/false"));
    execute_active(&always_true, Args::new(), &Vars::new())?;
    match execute_active(&always_false, Args::new(), &Vars::new()) {
        Err(e) => println!(
            "{} {}",
            Green.paint("/bin/false returned: "),
            Green.paint(e.to_string())
        ),
        _ => return Err(ApplyError::Error(String::from("OK not expected"))),
    }
    execute_active(
        &VirtualFile::in_memory_shell("echo hello".into()),
        Args::new(),
        &Vars::new(),
    )?;
    Ok(())
}

fn execute_inactive(script: &VirtualFile, args: Args, vars: &Vars) -> Result<(), ApplyError> {
    //        let exe_path = exectable_full_path(cmd)?;
    let cli = format!("{:?} {} {:?}", vars, script, args);
    log_cmd_action("run", Verb::WOULD, cli);
    Ok(())
}
fn execute_active(script: &VirtualFile, args: Vec<String>, vars: &Vars) -> Result<(), ApplyError> {
    let o = script.as_executable()?;
    let mut ps = Command::new(o.path());
    debug!("execute_active {:?}", ps);
    if args.len() > 0 {
        ps.args(args);
    }
    if vars.len() > 0 {
        ps.envs(vars);
    }
    let output = ps
        .output()
        .map_err(|e| ApplyError::ExecError(format!("execute_active output: {:?} {:?} {:?}", o.path(), script, e)))?;
    println!("{} {}", Green.paint("LIVE: run "), format!("{}", script));
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
                Err(ApplyError::NotZeroExit(n))
            }
        }
        None => Err(ApplyError::CmdExitedPrematurely),
    }
}

fn execute_interactive(script: &VirtualFile, args: Args, vars: &Vars) -> Result<(), ApplyError> {
    match ask(&format!("run (y/n): {}", script)) {
        'n' => {
            println!("{} {}", Yellow.paint("WOULD: run "), script);
            Ok(())
        }
        'y' => execute_active(script, args, &vars),
        _ => execute_interactive(script, args, vars),
    }
}

pub fn execute(mode: Mode, cmd: &VirtualFile, args: Args, vars: &Vars) -> Result<(), ApplyError> {
    match mode {
        Mode::Interactive => execute_interactive(cmd, args, vars),
        Mode::Passive => execute_inactive(cmd, args, vars),
        Mode::Active => execute_active(cmd, args, vars),
    }
}

pub fn do_action<'g>(mode: Mode, vars: Vars, action: Action) -> Result<(), ApplyError> {
    match action {
        Action::Template(template_file_name, output_file_name) => {
            let template_file = SrcFile::new(template_file_name);
            let output_file = DestFile::new(mode, PathBuf::from(output_file_name));
            process_template_file(mode, vars, &template_file, &output_file).map(|_diff_status| ())
        }
        Action::Execute(cmd, args) => {
            debug!("do_action execute {:?} {:?} {:?}", mode, cmd, args);
            execute(mode, &cmd, args, &vars)
        }
        Action::Error(ae) => Err(ae),
        Action::None => Ok(()),
    }
}

#[test]
fn test_do_action() -> Result<(), ApplyError> {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace")).try_init();
    let mut vars: Vars = Vars::new();
    vars.insert("value".into(), "FILLED".into());
    let template = Action::Template(
        VirtualFile::InMemory(String::from("key=@@value@@")),
        String::from("key_unit_test.txt"),
    );
    do_action(Mode::Passive, vars, template)?;
    // cat key_unit_test.txt  should be key=FILLED
    Ok(())
}

pub(crate) fn dryrun(input_list_vec: Iter<String>, mode: Mode) -> Result<(), ApplyError> {
    debug!("dryrun {:?}", mode);
    let mut vars = Vars::new();
    let mut input_list = input_list_vec.collect::<VecDeque<_>>();
    while let Some(input) = input_list.pop_front() {
        let t: Type = parse_type(input);
        debug!("type {:?}", t);
        let action = match t {
            Type::Template => {
                let infile = String::from(
                    input_list
                        .pop_front()
                        .expect("expected template: tp template output"),
                );
                let outfile = String::from(
                    input_list
                        .pop_front()
                        .expect("expected output: tp template output"),
                );
                if infile.starts_with("data:") {
                    Action::Template(VirtualFile::InMemory(infile[5..].into()), outfile)
                } else {
                    Action::Template(VirtualFile::FsPath(PathBuf::from(infile)), outfile)
                }
            }
            Type::Variable => {
                if let Some(k) = input_list.pop_front() {
                    debug!("k {}", k);
                    if let Some(v) = input_list.pop_front() {
                        vars.insert(k.into(), v.into());
                        debug!("v {}", v);
                        Action::None
                    } else {
                        Action::Error(ApplyError::ExpectedArg( format!("value for {}", k)))
                    }
                } else {
                    Action::Error(ApplyError::ExpectedArg("key".into()))
                }
            },
            Type::Execute => match input_list.pop_front() {
                None => Action::Error(ApplyError::ExpectedArg("expected execute path".into())),
                Some(cmd) => {
                    let exe = exectable_full_path(cmd)?;
                    debug!("exe {:?}", exe);
                    let script = VirtualFile::FsPath(exe);
                    let mut args = Args::new();
                    for e in input_list.split_off(input_list.len()) {
                        args.push(e.to_string());

                    }
                    debug!("args {:?}", args);
                    Action::Execute(script, args)
                }
            },
            Type::Unknown => {
                println!("{} {}", Red.paint("Unknown type:"), Red.paint(input));
                Action::Error( ApplyError::ExpectedArg( format!("Unknown type: {}", input)))
            }
        };
        //debug!("vars {:#?}", &vars);
        debug!("action {:#?}", action);
        do_action(mode, vars.clone(), action)?;
    }
    Ok(())
}
