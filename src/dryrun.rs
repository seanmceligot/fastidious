use crate::cmd::Args;
use crate::cmd::Vars;
use crate::cmd::VirtualFile;
use crate::files::Mode;
use crate::passive::log_cmd_action;
use crate::passive::Verb;
use ansi_term::Colour::{Green, Red, Yellow};
use applyerr::ApplyError;
use cmd::exectable_full_path;
use diff::create_or_diff;
use diff::diff;
use diff::update_from_template;
use diff::DiffStatus;
use env_logger::Env;
use files::DestFile;
use files::GenFile;
use files::SrcFile;
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
use template::{generate_recommended_file, replace_line, replace_line2, ChangeString};
use userinput::ask;

#[derive(Debug, Clone, Copy)]
pub enum ActionResult {
    Applied,
    Skipped,
    AlreadyApplied,
    Created,
}
impl From<DiffStatus> for ActionResult {
    fn from(ds: DiffStatus) -> Self {
        match ds {
            DiffStatus::NoChanges => Self::AlreadyApplied,
            DiffStatus::NewFile => Self::Applied,
            DiffStatus::Changed(_) => Self::Applied,
            DiffStatus::Unsupported => Self::Skipped, // TODO: handle this
            DiffStatus::Failed => Self::Skipped,      // TODO: handle this
        }
    }
}
fn process_template_file(
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
    let echo_hello = VirtualFile::in_memory_shell("echo hello".into());
    execute_active(
        &echo_hello,
        Args::new(),
        &Vars::new(),
    )?;
    Ok(())
}

fn execute_inactive(
    script: &VirtualFile,
    args: Args,
    vars: &Vars,
) -> Result<ActionResult, ApplyError> {
    //        let exe_path = exectable_full_path(cmd)?;
    let filled_args = replace_all(&args, vars)?;
    let cli = format!("{:?} {} {:?}", vars, script, filled_args);
    log_cmd_action("run", Verb::Would, cli);
    Ok(ActionResult::Skipped)
}
fn replace_all(args: &[String], vars: &Vars) -> Result<Vec<String>, ApplyError> {
    let filled_args: Vec<String> = args
        .iter()
        .map(|a| replace_line2(vars, a))
        .collect::<Result<Vec<String>, ApplyError>>()?;
    debug!("{:?}", filled_args);
    Ok(filled_args)
}
fn execute_active(
    script: &VirtualFile,
    args: Vec<String>,
    vars: &Vars,
) -> Result<ActionResult, ApplyError> {
    let o = script.as_executable()?;
    let mut ps = Command::new(o.path());
    debug!("o {:?}", o.path());
    debug!("execute_active {:?}", ps);
    if !args.is_empty() {
        let filled_args = replace_all(&args, vars)?;
        debug!("execute_active filled_args {:?}", filled_args);
        ps.args(filled_args);
    }

    //ps.envs(vars);
    let r = ps.output();
    let output = r.map_err(|e| {
        ApplyError::ExecError(format!(
            "execute_active execute failed: {:?} {:?} {:?}",
            o.path(),
            script,
            e
        ))
    })?;
    println!("{} {}", Green.paint("LIVE: run "), script);
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
                Ok(ActionResult::Applied)
            } else {
                Err(ApplyError::NotZeroExit(n))
            }
        }
        None => Err(ApplyError::CmdExitedPrematurely),
    }
}

fn execute_interactive(
    script: &VirtualFile,
    args: Args,
    vars: &Vars,
) -> Result<ActionResult, ApplyError> {
    let filled_args = replace_all(&args, vars)?;
    let strargs = filled_args.join(" ");
    match ask(&format!("run (y/n): {} {}", script, strargs)) {
        'n' => {
            println!("{} {} {}", Yellow.paint("SKIP: run "), script, strargs);
            Ok(ActionResult::Skipped)
        }
        'y' => execute_active(script, args, vars),
        _ => execute_interactive(script, args, vars),
    }
}

pub fn execute(
    mode: Mode,
    cmd: &VirtualFile,
    args: Args,
    vars: &Vars,
) -> Result<ActionResult, ApplyError> {
    match mode {
        Mode::Interactive => execute_interactive(cmd, args, vars),
        Mode::Passive => execute_inactive(cmd, args, vars),
        Mode::Active => execute_active(cmd, args, vars),
    }
}

#[test]
fn test_do_action() -> Result<(), ApplyError> {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace")).try_init();
    let mut vars: Vars = Vars::new();
    vars.insert("value".into(), "FILLED".into());
    let src = SrcFile::new(VirtualFile::InMemory(String::from("key=@@value@@")));
    let dest = DestFile::new(PathBuf::from("key_unit_test.txt"));
    process_template_file(Mode::Passive, vars, &src, &dest)?;
    // cat key_unit_test.txt  should be key=FILLED
    Ok(())
}
pub(crate) fn do_template(
    mode: Mode,
    vars: Vars,
    maybe_data: Option<String>,
    maybe_in: Option<PathBuf>,
    output_file: DestFile,
) -> Result<DiffStatus, ApplyError> {
    let infile = match maybe_data {
        Some(data) => VirtualFile::InMemory(data),
        None => VirtualFile::FsPath(maybe_in.unwrap()), // TODO: check unwrap
    };
    debug!("vars {:#?}", vars);

    let template_file = SrcFile::new(infile);
    process_template_file(mode, vars, &template_file, &output_file)
}
pub(crate) fn dryrun(
    mode: Mode,
    vars: Vars,
    cmd_line: Vec<String>,
) -> Result<ActionResult, ApplyError> {
    let cmd_exe = &cmd_line[0];
    let cmd_args = &cmd_line[1..];
    debug!("dryrun {:?}", mode);
    let exe = exectable_full_path(cmd_exe)?;
    debug!("exe {:?}", exe);
    let script = VirtualFile::FsPath(exe);
    let mut args = Args::new();
    for a in cmd_args {
        args.push(a.to_string());
    }
    debug!("args {:?}", args);
    debug!("do_action execute {:?} {:?} {:?}", mode, script, args);
    execute(mode, &script, args, &vars)
}
