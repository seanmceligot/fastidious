use log::trace;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;

use crate::applyerr::ApplyError;
use crate::cmd::OpenFileHolder;
use crate::cmd::VirtualFile;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Active,
    Passive,
    Interactive,
}
#[derive(Debug)]
pub struct SrcFile {
    path: VirtualFile,
}

impl SrcFile {
    pub fn new(path: VirtualFile) -> Self {
        Self { path }
    }
    pub fn open(&self) -> Result<OpenFileHolder, ApplyError> {
        trace!("SrcFile::open {:?}", self.path);
        self.path.as_readable()?.open()
    }
}

#[derive(Debug)]
pub struct DestFile {
    path: PathBuf,
}
impl DestFile {
    pub fn new(p: PathBuf) -> Self {
        DestFile { path: p }
    }
    pub fn _exists(&self) -> bool {
        self.path.exists()
    }
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
#[derive(Debug)]
pub struct GenFile {
    path: PathBuf,
}
impl GenFile {
    pub fn new() -> Result<Self, ApplyError> {
        let path = PathBuf::from(format!("{}.gen.tmp", rand::random::<u32>()));
        Ok(Self { path })
    }
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
    pub fn open(&self) -> Result<File, ApplyError> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.path.clone())
            .map_err(|e| ApplyError::FileWriteError(format!("Gen::open {:?} {:?}", self.path, e)))
    }
    //pub fn open(&self) -> std::fs::File {}
}
impl fmt::Display for SrcFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.path)
    }
}

impl fmt::Display for DestFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl fmt::Display for GenFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}
impl AsRef<OsStr> for DestFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
/*
impl AsRef<OsStr> for SrcFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
 */
impl AsRef<OsStr> for GenFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
