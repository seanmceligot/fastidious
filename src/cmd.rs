
use std::{collections::HashMap, env, fs::{File, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}, process::Command};
use ansi_term::Colour::{Green, Red, Yellow};
use tempfile::NamedTempFile;
use crate::applyerr::ApplyError;

#[derive(Debug)]
pub enum Script {
    FsPath(PathBuf),
    InMemory(String)
}
impl Script {
    fn open_readonly(&self) -> Result<OpenFileHolder,ApplyError> {
        match (self) {
            Script::FsPath(path) => {
                let maybe_file =OpenOptions::new().read(true).open(path);
                let f = maybe_file.map_err(|e| 
                    ApplyError::FileReadError(format!("read error {:?} {:?}", path, e)))?;
                Ok(OpenFileHolder::Perm(f, path.to_path_buf()))
            },
            Script::InMemory(source) => {
                let mut t = tempfile::NamedTempFile::new().unwrap();
                debug!("contents: {}", source);
                t.write_all(source.as_bytes()).
                    map_err(|e|ApplyError::FileWriteError(format!("{:?} {:?}", t.path(), e)))?;
                Ok(OpenFileHolder::Temp(t))
            }
        }
    }
}
enum OpenFileHolder {
    Temp(NamedTempFile),
    Perm(File, PathBuf) 
}
impl OpenFileHolder {
    fn file(&self) -> &File {
        match self {
            OpenFileHolder::Temp(named) => named.as_file(),
            OpenFileHolder::Perm(f, _p) => f
        }
    }
    fn path(&self) -> &Path {
        match self {
            OpenFileHolder::Temp(named) => named.path(),
            OpenFileHolder::Perm(_f,p) => p
        }

    }
}
pub fn cmdline(cmd: String, args: Vec<&str>) -> String {
    let mut full = vec![cmd.as_str()];
    full.append(&mut args.to_vec());
    full.join(" ")
}

pub fn exectable_full_path(prg: &str) -> Result<PathBuf, ApplyError> {
    let maybe_prg: which::Result<PathBuf> = which::which(prg);
    exectable_full_path_which(prg, maybe_prg)
}
fn exectable_full_path_which(
    prg: &str,
    maybe_prg: which::Result<PathBuf>,
) -> Result<PathBuf, ApplyError> {
    match maybe_prg {
        Ok(prg_path) => Ok(prg_path),
        Err(_e) => Err(ApplyError::CommandNotFound(String::from(prg))),
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
    let script_file = script.open_readonly()?;
    Ok(execute_script_file(script_file.path(), vars)?)
}
