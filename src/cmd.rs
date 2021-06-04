
use dryrunerr::DryRunError;
use std::{collections::HashMap, env, io::{self, Write}, path::{Path, PathBuf}, process::Command};
use ansi_term::Colour::{Green, Red, Yellow};

use crate::applyerr::ApplyError;
pub enum Script {
    FsPath(PathBuf),
    InMemory(String)
}

pub fn cmdline(cmd: String, args: Vec<&str>) -> String {
    let mut full = vec![cmd.as_str()];
    full.append(&mut args.to_vec());
    full.join(" ")
}

pub fn exectable_full_path(prg: &str) -> Result<PathBuf, DryRunError> {
    let maybe_prg: which::Result<PathBuf> = which::which(prg);
    exectable_full_path_which(prg, maybe_prg)
}
fn exectable_full_path_which(
    prg: &str,
    maybe_prg: which::Result<PathBuf>,
) -> Result<PathBuf, DryRunError> {
    match maybe_prg {
        Ok(prg_path) => Ok(prg_path),
        Err(_e) => Err(DryRunError::CommandNotFound(String::from(prg))),
    }
}
pub(crate) fn execute_script_file(cmdpath: &Path,  vars: HashMap<String, String>) -> Result<(), ApplyError> {
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
pub(crate) fn execute_script(script: &Script,  vars: HashMap<String, String>) -> Result<(), ApplyError> {
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
