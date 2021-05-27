extern crate libc;
use dryrun::userinput::ask;
use std::path::Path;
//use std::path::PathBuf;
use dryrun::err::{log_path_action, DryRunError, Verb::SKIPPED};
use dryrun::Mode;
use std::ffi::CString;

//#[cfg(not(test))]
use log::trace;
//#[cfg(test)]
//use std::{println as trace};

#[test]
fn test_can() {
    assert_eq!(
        can_create_dir_maybe(Path::new("/root/test").parent()).is_err(),
        true
    ); // /root
    assert_eq!(
        can_create_dir_maybe(Path::new("./Cargo.toml").parent()).is_ok(),
        true
    ); // ./
    assert_eq!(can_write_file(Path::new("tmp.txt")).is_ok(), true);
    assert_eq!(can_write_file(Path::new("./tmp.txt")).is_ok(), true);
    assert_eq!(can_create_dir(Path::new(".")).is_ok(), true);
}

//pub fn assert_nonempty_path(path: &Path) -> Result<(), DryRunError> { match path { None => Err(DryRunError::PathEmpty), _ => Ok(()) } }

fn access_w(path: &Path) -> bool {
    let cstr = CString::new(path.display().to_string()).unwrap();
    unsafe {
        matches!(libc::faccessat(libc::AT_FDCWD, cstr.as_ptr(), libc::W_OK, libc::AT_EACCESS) as isize, 0)
    }
}
fn access_x(path: &Path) -> bool {
    let cstr = CString::new(path.display().to_string()).unwrap();
    unsafe {
        matches!(libc::faccessat(libc::AT_FDCWD, cstr.as_ptr(), libc::X_OK, libc::AT_EACCESS) as isize, 0)
    }
}
pub fn can_write_file(path: &Path) -> Result<&Path, DryRunError> {
    trace!("can_write_file{:?}", path);
    if path.exists() {
        if access_w(path) {
            Ok(path)
        } else {
            Err(DryRunError::InsufficientPrivileges(
                path.display().to_string(),
            ))
        }
    } else {
        can_write_dir_maybe(path.parent())
    }
}
pub fn can_write_dir_maybe(maybe_dir: Option<&Path>) -> Result<&Path, DryRunError> {
    trace!("can_write_dir_maybe {:?}", maybe_dir);
    match maybe_dir {
        Some(dir) => can_write_dir(dir),
        None => Err(DryRunError::PathNotFound0),
    }
}
pub fn can_write_dir(dir: &Path) -> Result<&Path, DryRunError> {
    trace!("can_write_dir{:?}", dir);
    if dir.file_name().is_none() {
        let pwd = Path::new(".");
        if access_w(pwd) {
            Ok(pwd)
        } else {
            Err(DryRunError::InsufficientPrivileges(
                pwd.display().to_string(),
            ))
        }
    } else if dir.exists() {
        if access_w(dir) {
            Ok(dir)
        } else {
            Err(DryRunError::InsufficientPrivileges(
                dir.display().to_string(),
            ))
        }
    } else {
        can_create_dir_maybe(dir.parent())
    }
}
pub fn can_create_dir_maybe(maybe_dir: Option<&Path>) -> Result<&Path, DryRunError> {
    trace!("can_create_dir_maybe {:?}", maybe_dir);
    match maybe_dir {
        Some(dir) => can_create_dir(dir),
        None => Err(DryRunError::PathNotFound0),
    }
}
pub fn can_create_dir(dir: &Path) -> Result<&Path, DryRunError> {
    trace!("can_create_dir{:?}", dir);
    if dir.exists() {
        if access_x(dir) {
            Ok(dir)
        } else {
            Err(DryRunError::InsufficientPrivileges(
                dir.display().to_string(),
            ))
        }
    } else {
        can_create_dir_maybe(dir.parent())
    }
}
pub fn create_dir_maybe(mode: Mode, maybe_dir: Option<&Path>) -> Result<&Path, DryRunError> {
    trace!("create_dir_maybe {:?}", maybe_dir);
    match maybe_dir {
        Some(dir) => create_dir(mode, dir),
        None => Err(DryRunError::PathNotFound0),
    }
}
pub fn create_dir(mode: Mode, dir: &Path) -> Result<&Path, DryRunError> {
    trace!("create_dir{:?}", dir);
    if dir.exists() {
        Ok(dir)
    } else {
        let ans = match mode {
            Mode::Passive => 'n',
            Mode::Active => 'y',
            Mode::Interactive => ask(format!("create directory {} (y/n)", dir.display()).as_str()),
        };
        match ans {
            'n' => match can_create_dir_maybe(dir.parent()) {
                Err(e) => Err(e),
                Ok(dir) => {
                    log_path_action("create dir", SKIPPED, dir);
                    Ok(dir)
                }
            },
            'y' => {
                println!("mkdir {}", dir.display());
                match std::fs::create_dir_all(dir) {
                    Err(e) => Err(DryRunError::IoError(e)),
                    Ok(_) => Ok(dir),
                }
            }
            _ => create_dir(mode, dir), //repeat the question
        }
    }
}
