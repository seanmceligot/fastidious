use crate::applyerr::ApplyError;
use crate::fs;
use ansi_term::Colour::{Green, Red, Yellow};
use env_logger::Env;
use std::fmt;
use std::fs::canonicalize;
use std::io::Read;
use std::os::unix::prelude::OpenOptionsExt;
use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub type Vars = HashMap<String, String>;

pub type Args = Vec<String>;

pub(crate) fn to_vars(v: Vec<String>) -> Vars {
    v.iter()
        .map(|s| s.split_once('='))
        .flatten()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<HashMap<_, _>>()
}
#[test]
fn test_vars() -> () {
    {
        let v = vec![
            "a=1".to_string(),
            "b=2".to_string(),
            "c=3".to_string(),
            "foobarred".to_string(),
            "d=4".to_string(),
        ];
        let vars = to_vars(v);
        assert_eq!(vars.get("b").unwrap(), "2");
    }
}
#[test]
fn test_virtual_file() -> Result<(), ApplyError> {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace")).try_init();
    let text = String::from("Hello");
    let vf = VirtualFile::InMemory(text.clone());
    let mut s = String::new();
    let r = vf.as_readable()?;
    let o = r.open()?;
    debug!("path {:?}", o.path());
    let n = o
        .file()
        .read_to_string(&mut s)
        .map_err(|e| ApplyError::FileReadError(format!("{:?} {}", o.path(), e)))?;
    debug!("src contains: size: {} file contents {}", n, s);
    assert_eq!(s, text);
    Ok(())
}

#[derive(Debug)]
pub enum VirtualFile {
    FsPath(PathBuf),
    InMemory(String),
}
pub struct ExecutableFile {
    path: PathBuf,
}
impl ExecutableFile {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
pub struct ReadableFile {
    path: PathBuf,
}
impl ReadableFile {
    pub fn open(&self) -> Result<OpenFileHolder, ApplyError> {
        let f = OpenOptions::new()
            .read(true)
            .open(self.path.clone())
            .map_err(|e| {
                ApplyError::FileReadError(format!("read error {:?} {:?}", self.path, e))
            })?;
        Ok(OpenFileHolder::Perm(f, self.path.clone()))
    }
}

impl VirtualFile {
    pub(crate) fn in_memory_shell(script: String) -> Self {
        let mut full_script = String::from("#! /bin/sh\n");
        full_script.push_str(script.as_str());
        Self::InMemory(full_script)
    }
    pub fn as_executable(&self) -> Result<ExecutableFile, ApplyError> {
        match self {
            VirtualFile::FsPath(p) => {
                fs::can_execute(p.clone())?;
                Ok(ExecutableFile { path: p.clone() })
            }
            VirtualFile::InMemory(source) => {
                let path = PathBuf::from(format!("{}.tmp.sh", rand::random::<u32>()));
                debug!("contents: {}", source);
                write_file(
                    OpenOptions::new()
                        .mode(0o755)
                        .write(true)
                        .create(true)
                        .truncate(true),
                    path.clone(),
                    source,
                )?;
                let fullpath = canonicalize(path).unwrap();
                Ok(ExecutableFile { path: fullpath })
            }
        }
    }
    pub fn as_readable(&self) -> Result<ReadableFile, ApplyError> {
        match self {
            VirtualFile::FsPath(p) => {
                fs::can_read_file(p.clone())?;
                Ok(ReadableFile { path: p.clone() })
            }
            VirtualFile::InMemory(source) => {
                let path = PathBuf::from(format!("r{}.tmp", rand::random::<u32>()));
                debug!("contents: {}", source);
                write_file(
                    OpenOptions::new()
                        .mode(0o0644)
                        .write(true)
                        .truncate(true)
                        .create(true),
                    path.clone(),
                    source,
                )?;
                Ok(ReadableFile { path })
            }
        }
    }
}
impl fmt::Display for VirtualFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VirtualFile::FsPath(p) => write!(f, "{:?}", p),
            VirtualFile::InMemory(s) => write!(f, "{}", s),
        }
    }
}

pub enum OpenFileHolder {
    Perm(File, PathBuf),
}
impl OpenFileHolder {
    pub(crate) fn file(&self) -> &File {
        match self {
            OpenFileHolder::Perm(f, _p) => f,
        }
    }
    pub(crate) fn path(&self) -> &PathBuf {
        match self {
            OpenFileHolder::Perm(_f, p) => p,
        }
    }
}
pub fn _cmdline(cmd: String, args: Vec<&str>) -> String {
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
fn write_file(
    options: &mut OpenOptions,
    path: PathBuf,
    source: &str,
) -> Result<ExecutableFile, ApplyError> {
    let mut f = options
        .open(path.clone())
        .map_err(|e| ApplyError::FileCreateError(format!("{:?} {:?}", path, e)))?;
    f.write_all(source.as_bytes())
        .map_err(|e| ApplyError::FileWriteError(format!("write_file {:?} {:?}", path, e)))?;
    Ok(ExecutableFile { path })
}

/*
pub(crate) fn execute_script(script: &VirtualFile,  vars: Vars) -> Result<(), ApplyError> {
    do_action(crate::files::Mode::Passive, vars, Action::Execute(script.clone(), Vec::new()))
    //Ok(execute_script_file(script_file.path(), vars)?)
}
 */
