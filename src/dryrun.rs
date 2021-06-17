
use ansi_term::Colour::{Green, Red, Yellow};
use cmd::exectable_full_path;
use diff::create_or_diff;
use diff::diff;
use diff::update_from_template;
use diff::DiffStatus;
use applyerr::log_cmd_action;
use applyerr::ApplyError;
use applyerr::Verb;
use env_logger::Env;
use template::{generate_recommended_file, replace_line, ChangeString};
use userinput::ask;
use files::DestFile;
use files::GenFile;
use files::Mode;
use files::SrcFile;
use getopts::Options;
use log::trace;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::slice::Iter;
use std::str;

use crate::cmd::Args;
use crate::cmd::Script;
use crate::cmd::Vars;
use crate::cmd::VirtualFile;

/*
#[derive(Debug)]
pub enum VirtualFile {
    FsPath(String),
    InMemory(String)
}
impl From<VirtualFile> for PathBuf {
    fn from(vf: VirtualFile) -> PathBuf {
        match vf {
            VirtualFile::FsPath(s) => PathBuf::from(s),
            VirtualFile::InMemory(source) => {
                let mut t = tempfile::NamedTempFile::new().unwrap();
                t.write_all(source.as_bytes()).unwrap();
                debug!("tmp template {:?}", t.path());
                match t.keep() {
                     Ok((_file,p)) =>  p,
                     Err(persist_error) => {
                         panic!("persist error: {}", persist_error.to_string())
                       }
                    }
            }
        }
    }
}
*/
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
    Execute(Script, Args),
    Error(String),
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
    let always_true = Script::FsPath(PathBuf::from("/bin/true"));
    let always_false = Script::FsPath(PathBuf::from("/bin/false"));
    execute_active(&always_true, Args::new(), &Vars::new())?;
    match execute_active(&always_false, Args::new(), &Vars::new()) {
        Err(e) => println!(
            "{} {}",
            Red.paint("/bin/false returned: "),
            Red.paint(e.to_string())
        ),
        _ => return Err(ApplyError::Error(String::from("OK not expected"))),
    }
    execute_active(&Script::InMemory("#! /bin/bash\necho hello".into()), Args::new(), &Vars::new())?;
    Ok(())
}

fn execute_inactive(script: &Script, args: Args, vars: &Vars) -> Result<(), ApplyError> {
    //        let exe_path = exectable_full_path(cmd)?;
            let cli = format!("{:?} {} {:?}", vars, script, args);
            log_cmd_action("run", Verb::WOULD, cli);
            Ok(())

}
fn execute_active(script: &Script, args: Vec<String>, vars: &Vars) -> Result<(), ApplyError> {
    let o = script.as_executable()?;
    let mut ps = Command::new(o.path());
    debug!("execute_active {:?}", ps);
    if args.len() > 0{
        ps.args(args);
    }
    if vars.len() > 0 {
        ps.envs(vars);
    }
    let output = ps.output().map_err(|e| ApplyError::ExecError(
        format!("{:?} {:?}", script, e)
    ))?;
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

fn execute_interactive(script: &Script, args: Args, vars: &Vars) -> Result<(), ApplyError> {
    match ask(&format!("run (y/n): {}", script)) {
        'n' => {
            println!("{} {}", Yellow.paint("WOULD: run "), script);
            Ok(())
        }
        'y' => execute_active(script, args, &vars),
        _ => execute_interactive(script,args, vars),
    }
}

pub fn execute(mode: Mode, cmd: &Script, args: Args, vars: &Vars) -> Result<(), ApplyError> {
    match mode {
        Mode::Interactive => execute_interactive(cmd, args, vars),
        Mode::Passive => execute_inactive(cmd, args, vars),
        Mode::Active => execute_active(cmd, args, vars),
    }
}

pub fn do_action<'g>(
    mode: Mode,
    vars: Vars,
    action: Action,
) -> Result<(), ApplyError> {
    match action {
        Action::Template(template_file_name, output_file_name) => {
            let template_file = SrcFile::new(template_file_name);
            let output_file = DestFile::new(mode, PathBuf::from(output_file_name));
            process_template_file(mode, vars, &template_file, &output_file)
            .map(|_diff_status|())
        }
        Action::Execute(cmd, args) => {
            debug!("do_action execute {:?} {:?} {:?}", mode, cmd, args);
            execute(mode, &cmd, args, &vars)
        }
        Action::Error(msg) => Err(ApplyError::Error(msg)),
        Action::None => Ok(()),
    }
}

#[test]
fn test_do_action() ->Result<(), ApplyError> {
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
fn expect_option<R>(a: Option<R>, emsg: &str) -> Result<R, ApplyError> {
    match a {
        Some(r) => Ok(r),
        None => {
            println!("{}", Red.paint(emsg));
            Err(ApplyError::Warn)
        }
    }
}

pub(crate) fn dryrun(mut input_list: Iter<String>, mode: Mode)  -> Result<(), ApplyError>{
    debug!("dryrun {:?}", mode);
    let mut vars = Vars::new();
    if let Some(input) = input_list.next() {
        let t: Type = parse_type(input);
        let action = match t {
            Type::Template => {
                let infile = String::from(
                    input_list
                        .next()
                        .expect("expected template: tp template output"),
                );
                let outfile = String::from(
                    input_list
                        .next()
                        .expect("expected output: tp template output"),
                );
                Action::Template(VirtualFile::FsPath(PathBuf::from(infile)), outfile)
            }
            Type::Variable => {
                match expect_option(input_list.next(), "expected key: v key value") {
                    Ok(k) => {
                        match expect_option(input_list.next(), "expected value: v key value") {
                            Ok(v) => {
                                vars.insert(k.into(), v.into());
                                Action::None
                            }
                            Err(_) => Action::Error(format!("expected variable value for {}", k)),
                        }
                    }
                    Err(e) => {
                        println!(
                            "Variable: {} {}",
                            Red.paint("error:"),
                            Red.paint(e.to_string())
                        );
                        Action::Error(String::from("expected variable key"))
                    }
                }
            }
            Type::Execute => {
                match input_list.next() {
                    None => Action::Error("expected execute path".into()),
                    Some(cmd) => {
                        let exe = exectable_full_path(cmd)?;
                        let script = Script::FsPath(exe);
                        let rest = input_list
                            .map(|s|s.clone())
                            .collect();
                        Action::Execute(script, rest)       
                    }
                }
            }
            Type::Unknown => {
                println!("{} {}", Red.paint("Unknown type:"), Red.paint(input));
                Action::Error(format!("Unknown type: {}", input))
            }
        };
        //debug!("vars {:#?}", &vars);
        debug!("action {:#?}", action);
        do_action(mode, vars.clone(), action)
    } else {
        Err(ApplyError::ExpectedArg("x t v"))
    }
}
