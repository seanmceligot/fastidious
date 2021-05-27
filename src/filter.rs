extern crate tempfile;
use cmd::exectable_full_path;
use dryrunerr::DryRunError;
use files::{GenFile, SrcFile};
use std::collections::HashMap;
use std::io;
use std::process::Command;
use std::process::Stdio;

#[cfg(test)]
mod tests {
    use super::*;
    use diff::create_or_diff;
    use diff::DiffStatus;
    use files::{DestFile, GenFile, Mode, SrcFile};
    use std::path::PathBuf;

    #[test]
    fn test_filter() {
        let _vars: HashMap<&str, &str> = HashMap::new();
        let _src = SrcFile::new(PathBuf::from("Cargo.toml"));
        //let dest = DestFile::new(Mode::Active, PathBuf::from("./UPCASE.toml"));
        //let args = vec![&String::from("a-Z"),&String::from("A-Z")];
        //let _gen = generate_filtered_file(&vars, &src, String::from("tr"), args ).expect("generate_filtered_file");
    }
}
// creates the tmp file for comparing to the dest file
pub fn generate_filtered_file<'a>(
    _vars: &'a HashMap<&str, &str>,
    src: &SrcFile,
    cmd: String,
    args: Vec<&'a String>,
) -> Result<GenFile, DryRunError> {
    let gen = GenFile::new();
    let cmdpath = exectable_full_path(&cmd)?;
    match Command::new(cmdpath)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn : {}", why),
        Ok(process) => {
            //let infile =  src.open()?;
            //let stdin = process.stdin.expect("could not get stdin");
            //let stdout = process.stdout.expect("could not get stdout");
            //let reader = BufReader::new(infile.unwrap());
            //let  outfile =  gen.open();
            io::copy(&mut src.open()?, &mut process.stdin.unwrap())?;
            io::copy(&mut process.stdout.unwrap(), &mut gen.open())?;

            Ok(gen)
        }
    }
}
