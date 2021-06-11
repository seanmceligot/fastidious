
use std::{collections::HashMap, env, fs::{File, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}, process::Command};
use ansi_term::Colour::{Green, Red, Yellow};
use seahorse::App;
use temp_file::{self, TempFile};
use crate::applyerr::ApplyError;
use std::io::Read;


#[test]
fn test_virtual_file() -> Result<(), ApplyError> {
    dotenv::dotenv().ok();
    env_logger::init();
    let text =String::from("Hello");
    let vf = VirtualFile::InMemory(text.clone());
    let mut s = String::new();
    let o = vf.open_readonly()?;
    debug!("path {:?}", o.path());
    match o.file().read_to_string(&mut s) {
        Err(why) => panic!("couldn't read src: {}", why),
        Ok(_) => {
            debug!("src contains: {} {}", s, s);
            assert_eq!(s, text);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum Script {
    FsPath(PathBuf),
    InMemory(String)
}
pub type VirtualFile = Script;
impl Script {
    pub fn open_readonly(&self) -> Result<OpenFileHolder,ApplyError> {
        match self {
            Script::FsPath(path) => {
                let maybe_file =OpenOptions::new().read(true).open(path);
                let f = maybe_file.map_err(|e| 
                    ApplyError::FileReadError(format!("read error {:?} {:?}", path, e)))?;
                Ok(OpenFileHolder::Perm(f, path.to_path_buf()))
            },
            Script::InMemory(source) => {
                let temp = temp_file::with_contents(source.as_bytes());
                debug!("contents: {}", source);
                let f = OpenOptions::new()
                    .read(true)
                    .open(temp.path())
                    .map_err(|e|ApplyError::FileReadError(format!("{:?} {:?}", temp.path(), e)))?;
                Ok(OpenFileHolder::Temp(f, temp))
            }
        }
    }
}
pub enum OpenFileHolder {
    Temp(File, TempFile),
    Perm(File, PathBuf) 
}
impl OpenFileHolder {
    pub(crate) fn file(&self) -> &File {
        match self {
            OpenFileHolder::Temp(f, _tf) => f,
            OpenFileHolder::Perm(f, _p) => f
        }
    }
    pub(crate) fn path(&self) -> &Path {
        match self {
            OpenFileHolder::Temp(_f, t) => t.path(),
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
