
use log::trace;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Active,
    Passive,
    Interactive,
}
#[derive(Debug)]
pub struct SrcFile {
    path: PathBuf,
}

impl SrcFile {
    pub fn new(p: PathBuf) -> SrcFile {
        SrcFile { path: p }
    }
    pub fn open(&self) -> Result<File, std::io::Error> {
        trace!("open path {:?}", self.path);
        OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&self.path)
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
    file: tempfile::NamedTempFile,
}
impl GenFile {
    pub fn new() -> GenFile {
        GenFile {
            file: tempfile::NamedTempFile::new().unwrap(),
        }
    }
    pub fn open(&self) -> &File {
        self.file.as_file()
    }
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

impl Default for GenFile {
    fn default() -> Self {
        GenFile::new()
    }
}

impl fmt::Display for SrcFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
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
impl AsRef<OsStr> for SrcFile {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}
impl AsRef<OsStr> for GenFile {
    fn as_ref(&self) -> &OsStr {
        self.path().as_os_str()
    }
}
