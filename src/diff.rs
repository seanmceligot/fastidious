use crate::applyerr::Verb;
use crate::fs::{can_create_dir, can_create_parent_dir, create_parent_dir};
use ansi_term::Colour;
use ansi_term::Colour::{Red, Yellow};
use applyerr::Verb::{LIVE, SKIPPED, WOULD};
use applyerr::{color_from_verb, ApplyError};
use cmd::exectable_full_path;
use files::Mode;
use files::{DestFile, GenFile, SrcFile};
use fs::can_write_file;
use log::debug;
use log::trace;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::vec::IntoIter;
use userinput::ask;

#[derive(Debug)]
pub enum DiffText {
    Text(Vec<u8>),
    Unsupported,
}

#[derive(Debug)]
pub enum DiffStatus {
    NoChanges,
    NewFile,
    Changed(DiffText),
    Unsupported,
    Failed,
}
impl fmt::Display for DiffText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiffText::Text(difftext) => {
                for u in difftext.iter() {
                    let ch = *u as char;
                    write!(f, "{}", ch)?;
                }
            }
            DiffText::Unsupported => {
                write!(f, "Unsupported")?;
            }
        }
        Ok(())
    }
}

pub fn log_template_action(
    action: &'static str,
    verb: Verb,
    template: &SrcFile,
    gen: &GenFile,
    dest: &DestFile,
) {
    let color: Colour = color_from_verb(verb);
    println!(
        "{}: {} {} [{}]  ->{}",
        color.paint(verb.to_string()),
        color.paint(action),
        color.paint(template.to_string()),
        color.paint(gen.to_string()),
        color.paint(dest.to_string())
    );
}

pub fn diff<'a>(path: PathBuf, path2: PathBuf) -> DiffStatus {
    if !path2.exists() {
        DiffStatus::NewFile
    } else if !path2.is_file() {
        DiffStatus::Unsupported
    } else {
        let output = Command::new("diff")
            .arg(path)
            .arg(path2)
            .output()
            .expect("diff failed");
        //io::stdout().write_all(&output.stdout).unwrap();
        match output.status.code().unwrap() {
            1 => DiffStatus::Changed(DiffText::Text(output.stdout.clone())),
            2 => DiffStatus::Failed,
            0 => DiffStatus::NoChanges,
            _ => DiffStatus::Failed,
        }
    }
}
pub fn create_or_diff(
    mode: Mode,
    template: &SrcFile,
    dest: &DestFile,
    gen: &GenFile,
) -> Result<DiffStatus, ApplyError> {
    debug!("create_or_diff: diff {:?} {:?}", gen, dest.path());
    diff(gen.path(), dest.path());
    match update_from_template(mode, template, gen, dest) {
        Ok(_) => Ok(diff(gen.path(), dest.path())),
        Err(e) => Err(e),
    }
}
pub fn update_from_template<'f>(
    mode: Mode,
    template: &'f SrcFile,
    gen: &'f GenFile,
    dest: &'f DestFile,
) -> Result<(), ApplyError> {
    trace!("update_from_template");
    trace!("dest {:?}", dest);

    can_write_file(dest.path())?;

    let status = diff(gen.path(), dest.path());
    match status {
        DiffStatus::NoChanges => {
            println!(
                "{} {}",
                Yellow.paint("NO CHANGE: "),
                Yellow.paint(dest.to_string())
            );
            Ok(())
        }
        DiffStatus::Failed => {
            debug!("diff failed '{}'", dest);
            Err(ApplyError::Error(format!("diff failed '{}'", dest)))
        }
        DiffStatus::NewFile => {
            debug!("create '{}'", dest);
            debug!("cp {:?} {:?}", gen, dest);
            match mode {
                Mode::Passive => create_passive(gen, dest, template),
                Mode::Active => copy_active(gen, dest, template),
                Mode::Interactive => copy_interactive(gen, dest, template),
            }
        }
        DiffStatus::Unsupported => match mode {
            Mode::Passive => {
                update_from_template_passive(DiffText::Unsupported, template, gen, dest)
            }
            Mode::Active => update_from_template_active(template, gen, dest),
            Mode::Interactive => {
                update_from_template_interactive(DiffText::Unsupported, template, gen, dest)
            }
        },
        DiffStatus::Changed(difftext) => match mode {
            Mode::Passive => update_from_template_passive(difftext, template, gen, dest),
            Mode::Active => update_from_template_active(template, gen, dest),
            Mode::Interactive => update_from_template_interactive(difftext, template, gen, dest),
        },
    }
}
fn create_passive(gen: &GenFile, dest: &DestFile, template: &SrcFile) -> Result<(), ApplyError> {
    info!("template {:?}", template);
    can_create_parent_dir(dest.path())?;
    can_create_parent_dir(gen.path())
}
fn copy_active(gen: &GenFile, dest: &DestFile, template: &SrcFile) -> Result<(), ApplyError> {
    create_parent_dir(Mode::Active, dest.path())?;
    log_template_action("create from template", LIVE, template, gen, dest);
    match std::fs::copy(gen.path(), dest.path()) {
        Err(e) => Err(ApplyError::CopyError(
            gen.path(),
            dest.path(),
            e.to_string(),
        )),
        Ok(_) => Ok(()),
    }
}
fn copy_interactive(gen: &GenFile, dest: &DestFile, _template: &SrcFile) -> Result<(), ApplyError> {
    // TODO: add vimdiff support
    // TODO: use ask and copy_passive
    let status = Command::new("cp")
        .arg("-i")
        .arg(gen)
        .arg(dest)
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    if status.success() {
        Ok(())
    } else {
        panic!("cp failed: {:?} -> {:?}", gen, dest)
    }
}
fn merge_into_template(
    template: &SrcFile,
    _gen: &GenFile,
    dest: &DestFile,
) -> Result<bool, ApplyError> {
    let template_file = template.open()?;
    let status = Command::new("vim")
        .arg("-d")
        .arg(dest)
        .arg(template_file.path())
        .status()
        .expect("failed to execute process");

    println!("with: {}", status);
    Ok(status.success())
}
fn exit_status_to_dryrun_error(r: std::io::Result<ExitStatus>) -> Result<(), ApplyError> {
    match r {
        Err(ioe) => Err(ApplyError::IoError(ioe)),
        Ok(status) => match status.code() {
            None => Err(ApplyError::CmdExitedPrematurely),
            Some(_status_code) => Ok(()),
        },
    }
}
fn merge_to_template_interactive(
    _template: &SrcFile,
    gen: &GenFile,
    dest: &DestFile,
) -> Result<(), ApplyError> {
    let exe = String::from("vim");
    let real_exe: PathBuf = exectable_full_path(&exe)?;
    let mut cmd1 = Command::new(real_exe);
    let cmd = cmd1.arg("-d").arg(gen).arg(dest);
    exit_status_to_dryrun_error(cmd.status())
}

fn update_from_template_passive(
    difftext: DiffText,
    template: &SrcFile,
    gen: &GenFile,
    dest: &DestFile,
) -> Result<(), ApplyError> {
    log_template_action("create from template", WOULD, template, gen, dest);
    println!("{}", difftext);
    Ok(())
}
fn update_from_template_active(
    template: &SrcFile,
    gen: &GenFile,
    dest: &DestFile,
) -> Result<(), ApplyError> {
    copy_active(gen, dest, template)
}
fn update_from_template_interactive(
    difftext: DiffText,
    template: &SrcFile,
    gen: &GenFile,
    dest: &DestFile,
) -> Result<(), ApplyError> {
    let ans = ask(&format!(
        "{}: {} {} (o)verwrite / (m)erge[vimdiff] / s(k)ip / (d)iff / merge to (t)emplate",
        "files don't match", gen, dest
    ));
    match ans {
        'd' => update_from_template_passive(difftext, template, gen, dest),
        'k' => {
            log_template_action("create from template", SKIPPED, template, gen, dest);
            Ok(())
        }
        't' => {
            merge_into_template(template, gen, dest)?;
            Ok(())
        }
        'm' => merge_to_template_interactive(template, gen, dest).map(|_status_code| ()),
        'o' => copy_active(gen, dest, template),
        _ => update_from_template(Mode::Interactive, template, &gen, dest),
    }
}
