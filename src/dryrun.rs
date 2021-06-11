
use ansi_term::Colour::{Green, Red, Yellow};
use cmd::cmdline;
use cmd::exectable_full_path;
use diff::create_or_diff;
use diff::diff;
use diff::update_from_template;
use diff::DiffStatus;
use applyerr::log_cmd_action;
use applyerr::ApplyError;
use applyerr::Verb;
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

use crate::cmd::Script;
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
enum Action {
    Template(VirtualFile, String),
    Execute(String),
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
    vars: &'t HashMap<&'_ str, &'_ str>,
    template: &SrcFile,
    dest: &DestFile,
) -> Result<DiffStatus, ApplyError> {
    let gen = generate_recommended_file(vars, template)?;
    create_or_diff(mode, template, dest, &gen)
}

#[test]
fn test_execute_active() -> Result<(), ApplyError> {
    execute_active("/bin/true")?;
    match execute_active("/bin/false") {
        Err(e) => println!(
            "{} {}",
            Red.paint("/bin/false returned: "),
            Red.paint(e.to_string())
        ),
        _ => return Err(ApplyError::Error(String::from("OK not expected"))),
    }
    execute_active("echo echo_ping")?;
    Ok(())
}

fn execute_inactive(raw_cmd: &str) -> Result<(), ApplyError> {
    let empty_vec: Vec<&str> = Vec::new();
    let v: Vec<&str> = raw_cmd.split(' ').collect();
    let (cmd, args): (&str, Vec<&str>) = match v.as_slice() {
        [] => ("", empty_vec),
        //[cmd] => (cmd, empty_vec),
        [cmd, args @ ..] => (cmd, args.to_vec()),
    };
    match cmd {
        "" => Err(ApplyError::ExpectedArg("x command")),
        _ => {
            trace!("{}", cmd);
            let exe_path = exectable_full_path(cmd)?;
            trace!("{:?}", exe_path);
            trace!("{:?}", args);
            let cli = cmdline(exe_path.display().to_string(), args);
            log_cmd_action("run", Verb::WOULD, cli);
            Ok(())
        }
    }
}

fn execute_active(cmd: &str) -> Result<(), ApplyError> {
    let parts: Vec<&str> = cmd.split(' ').collect();
    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .expect("cmd failed");
    println!("{} {}", Green.paint("LIVE: run "), Green.paint(cmd));
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

fn execute_interactive(cmd: &str) -> Result<(), ApplyError> {
    match ask(&format!("run (y/n): {}", cmd)) {
        'n' => {
            println!("{} {}", Yellow.paint("WOULD: run "), Yellow.paint(cmd));
            Ok(())
        }
        'y' => execute_active(cmd),
        _ => execute_interactive(cmd),
    }
}

fn execute(mode: Mode, cmd: &str) -> Result<(), ApplyError> {
    match mode {
        Mode::Interactive => execute_interactive(cmd),
        Mode::Passive => execute_inactive(cmd).map(|_pathbuf| ()),
        Mode::Active => execute_active(cmd),
    }
}

fn do_action<'g>(
    mode: Mode,
    vars: &'g HashMap<&'g str, &'g str>,
    action: Action,
) -> Result<(), ApplyError> {
    match action {
        Action::Template(template_file_name, output_file_name) => {
            let template_file = SrcFile::new(template_file_name);
            let output_file = DestFile::new(mode, PathBuf::from(output_file_name));

            match process_template_file(mode, &vars, &template_file, &output_file) {
                Err(e) => {
                    println!(
                        "do_action: {} {}",
                        Red.paint("error:"),
                        Red.paint(e.to_string())
                    );
                    Err(e)
                }
                _ => Ok(()),
            }
        }
        Action::Execute(cmd) => {
            let the_cmd = match replace_line(vars, cmd.clone())? {
                ChangeString::Changed(new_cmd) => new_cmd,
                ChangeString::Unchanged => cmd,
            };
            match execute(mode, &the_cmd) {
                Ok(()) => Ok(()),
                Err(e) => {
                    println!(
                        "do_action: {} {}",
                        Red.paint("error:"),
                        Red.paint(e.to_string())
                    );
                    Err(e)
                }
            }
        }
        Action::Error(msg) => Err(ApplyError::Error(msg)),
        Action::None => Ok(()),
    }
}

#[test]
fn test_do_action() {
    let mut vars: HashMap<&str, &str> = HashMap::new();
    vars.insert("value", "unit_test");
    let template = Action::Template(
        VirtualFile::InMemory(String::from("key=@@value@@")),
        String::from("/tmp/key_unit_test.txt"),
    );
    match do_action(Mode::Passive, &vars, template) {
        Ok(_) => {}
        Err(_) => std::process::exit(1),
    }
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

pub(crate) fn dryrun(mut input_list: Iter<String>, mode: Mode) {
    debug!("dryrun {:?}", mode);
    let mut vars: HashMap<&str, &str> = HashMap::new();
    {
        while let Some(input) = input_list.next() {
            let t: Type = parse_type(input);
            let mut cmd = String::new();
            let mut cmdargs: Vec<String> = Vec::new();
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
                                    vars.insert(k, v);
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
                    #[allow(clippy::while_let_on_iterator)]
                    while let Some(input) = input_list.next() {
                        if cmd.is_empty() {
                            cmd.push_str(&input.to_string());
                        } else {
                            cmd.push(' ');
                            cmd.push_str(&input.to_string());
                        }
                    }
                    //let cmd_str: &str = cmd.as_str();
                    Action::Execute(cmd)
                }
                Type::Unknown => {
                    println!("{} {}", Red.paint("Unknown type:"), Red.paint(input));
                    Action::Error(format!("Unknown type: {}", input))
                }
            };
            //debug!("vars {:#?}", &vars);
            debug!("action {:#?}", action);
            match do_action(mode, &vars, action) {
                Ok(a) => a,
                Err(_) => std::process::exit(1),
            }
        }
    }
}
