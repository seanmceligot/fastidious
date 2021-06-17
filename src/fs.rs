extern crate libc;
use env_logger::Env;
use seahorse::App;
use userinput::ask;
use std::{env, path::{PathBuf}};
//use std::path::PathBuf;
use applyerr::{log_path_action, ApplyError, Verb::SKIPPED};
use files::Mode;
use std::ffi::CString;

//#[cfg(not(test))]
use log::trace;
//#[cfg(test)]
//use std::{println as trace};

#[test]
fn test_can() -> Result<(), ApplyError> {
    
    assert!(can_create_parent_dir(PathBuf::from("/root/test")).is_err());
    can_create_parent_dir(PathBuf::from("./Cargo.toml"))?;
    assert!(can_write_file(PathBuf::from("tmp.txt")).is_ok());
    assert!(can_write_file(PathBuf::from("./tmp.txt")).is_ok());
    assert!(can_create_dir(PathBuf::from(".")).is_ok());
    assert!(can_read_file(PathBuf::from("Cargo.toml")).is_ok());
    assert!(can_execute(PathBuf::from("/usr/bin/true")).is_ok() );
    Ok(())
}

//pub fn assert_nonempty_path(path: PathBuf) -> Result<(), ApplyError> { match path { None => Err(ApplyError::PathEmpty), _ => Ok(()) } }

fn access_w(path: PathBuf) -> Result<(), ApplyError> {
    let cstr = CString::new(path.display().to_string()).unwrap();
    unsafe {
        if matches!(libc::faccessat(libc::AT_FDCWD, cstr.as_ptr(), libc::W_OK, libc::AT_EACCESS) as isize, 0) {
            Ok(())
        } else {
            Err(ApplyError::InsufficientPrivileges(format!("write {:?}", path)))
        }
    }
}
fn access_r(path: PathBuf) -> Result<(), ApplyError> {
    let cstr = CString::new(path.display().to_string()).unwrap();
    unsafe {
        if matches!(libc::faccessat(libc::AT_FDCWD, cstr.as_ptr(), libc::R_OK, libc::AT_EACCESS) as isize, 0) {
            Ok(())
        } else {
            Err(ApplyError::InsufficientPrivileges(format!("read {:?}", path)))

        }
    }
}

fn access_x(path: PathBuf) -> Result<(), ApplyError> {
    let cstr = CString::new(path.display().to_string()).unwrap();
    unsafe {
        if matches!(libc::faccessat(libc::AT_FDCWD, cstr.as_ptr(), libc::X_OK, libc::AT_EACCESS) as isize, 0) {
            Ok(()) 
        } else {
            Err(ApplyError::InsufficientPrivileges(format!("execute {:?}", path)))
        }
    }
}
pub fn can_write_file(path: PathBuf) -> Result<(), ApplyError> {
    trace!("can_write_file{:?}", path);
    if path.exists() {
        access_w(path)
    } else {
        can_write_to_parent_dir(path)
    }
}
pub fn can_write_to_parent_dir(path: PathBuf) -> Result<(), ApplyError> {
    trace!("can_write_to_parent_dir {:?}", path);
    match path.parent() {
        Some(dir) =>  can_write_dir(dir.to_path_buf()),
        None => {
            // relative, use current_dir
            let pwd = env::current_dir().map_err(
                |e| ApplyError::PathNotFound(".".into()))?;     
            can_write_dir(pwd)
        }
    }
}
pub fn can_write_dir(dir: PathBuf) -> Result<(), ApplyError> {
    trace!("can_write_dir{:?}", dir);
    if dir.exists() {
        access_w(dir)
    } else {
        // backtrack to find an existing directory to check permissions
        can_create_parent_dir(dir)
    }
}
pub fn can_create_parent_dir(child: PathBuf) -> Result<(), ApplyError> {
    trace!("can_create_parent_dir {:?}", child);
    match child.parent() {
        None => can_create_dir(PathBuf::from(".")),
        Some(parent) => can_create_dir(parent.to_path_buf())
    }
}
pub fn can_execute(path: PathBuf) -> Result<(), ApplyError> {
    trace!("can_execute{:?}", path);
    let r =if path.exists() {
        if !path.is_dir() {
           access_x(path)
        } else {
            Err(ApplyError::NotAFile(path))
        }
    } else {
        Err(ApplyError::PathNotFound(format!("{:?}", path)))
    };
    r
}
pub fn can_read_file(path: PathBuf) -> Result<(), ApplyError> {
    trace!("can_read_file{:?}", path);
    let r =if path.exists() {
        if !path.is_dir() {
           access_r(path)
        } else {
            Err(ApplyError::NotAFile(path))
        }
    } else {
        Err(ApplyError::PathNotFound(format!("{:?}", path)))
    };
    r
}


pub fn can_create_dir(dir: PathBuf) -> Result<(), ApplyError> {
    trace!("can_create_dir{:?}", dir);
    if dir.exists() {
        access_x(dir)
    } else {
        can_create_parent_dir(dir)
    }
}
pub fn create_parent_dir(mode: Mode, child: PathBuf) -> Result<(), ApplyError> {
    trace!("create_dir_maybe {:?}", child);
    match child.parent() {
        Some(dir) => create_dir(mode, dir.to_path_buf()),
        None => Err(ApplyError::InsufficientPrivileges(
            format!("create parent dir {:?}",child)
        ))
    }
}
pub fn create_dir(mode: Mode, dir: PathBuf) -> Result<(), ApplyError> {
    trace!("create_dir{:?}", dir);
    if dir.exists() {
        Ok(())
    } else {
        let ans = match mode {
            Mode::Passive => 'n',
            Mode::Active => 'y',
            Mode::Interactive => ask(format!("create directory {} (y/n)", dir.display()).as_str()),
        };
        match ans {
            'n' => can_create_parent_dir(dir),
            'y' => {
                println!("mkdirs {:?}", dir);
                std::fs::create_dir_all(dir.clone()).map_err(
                    |e| ApplyError::InsufficientPrivileges(
                        format!("create_dir_all {:?} {:?}", dir.clone(), e)))
            }
            _ => create_dir(mode, dir), //repeat the question
        }
    }
}
