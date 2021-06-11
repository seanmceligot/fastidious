
use log::trace;
use temp_file::TempFile;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;

use crate::applyerr::ApplyError;
use crate::cmd::OpenFileHolder;
use crate::cmd::Script;
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
    pub fn new(path: VirtualFile) -> SrcFile {
        SrcFile { path: path }
    }
    pub fn open(&self) -> Result<OpenFileHolder, ApplyError> {
        trace!("SrcFile::open {:?}", self.path);
        self.path.open_readonly()
    }
}

#[derive(Debug)]
pub struct DestFile {
    mode: Mode,
    path: PathBuf,
}
impl DestFile {
    pub fn new(m: Mode, p: PathBuf) -> DestFile {
        DestFile { mode: m, path: p }
    }
    pub fn _exists(&self) -> bool {
        self.path.exists()
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}
#[derive(Debug)]
pub struct GenFile {
    file: temp_file::TempFile,
}
impl GenFile {
    pub fn new() -> Result<GenFile, ApplyError> {
        let tf = TempFile::new()
            .map_err(|e|
                ApplyError::FileWriteError(
                    format!("{:?} {:?}", "gen", e)))?;

        Ok(GenFile { file:tf })
    }
    pub fn open(&self) -> Result<File, ApplyError> {
        debug!("GenFile::open {:?}", self.file.path());
        let f = OpenOptions::new().write(true).create(true).open(self.file.path())
            .map_err(|e|ApplyError::FileWriteError(format!("{:?} {:?}", "gen", e)))?;
        Ok(f)
    }
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}
/*
impl Default for GenFile {
    fn default() -> Self {
        GenFile::new()
    }
} */

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
        self.path().as_os_str()
    }
}
