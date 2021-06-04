use dryrunerr::DryRunError;
use files::{GenFile, SrcFile};
use log::trace;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::ops::Range;
use regex::Regex;
use regex::Match;

#[test]
fn test_regex() {
    match match_line(String::from("a@@foo@@").as_str()) {
        Some(("foo", _)) => {}
        Some((_, _)) => panic!("fail"),
        None => panic!("expected Template"),
    }
    match match_line(String::from("@@foo@@a").as_str()) {
        Some(("foo", _)) => {}
        Some((_, _)) => panic!("fail"),
        None => panic!("expected Template"),
    }
    match match_line(String::from("@@foo@@").as_str()) {
        Some(("foo", _)) => {}
        Some((_, _)) => panic!("fail"),
        None => panic!("expected Template"),
    }
}
fn match_line<'a>(line: &'a str) -> Option<(&'a str, Range<usize>)> {
    let re = Regex::new(r"@@(?P<k>[A-Za-z0-9_\.-]*)@@").unwrap();
    match re.captures(line) {
        Some(cap) => {
            let all: Match = cap.get(0).unwrap();
            let k: Match = cap.name("k").unwrap();
            let key = k.as_str();
            Some((key, all.range()))
        }
        None => None,
    }
}
pub enum ChangeString {
    Changed(String),
    Unchanged,
}
pub fn replace_line(vars: &HashMap<&str, &str>, line: String) -> Result<ChangeString, DryRunError> {
    match match_line(line.as_str()) {
        Some((key, range)) => {
            let mut new_line: String = String::new();
            let v = vars.get(key);
            trace!("key {}", key);
            trace!("val {:?}", v);
            trace!("line {:?}", line);
            let before: &str = &line[..range.start];
            let after: &str = &line[range.end..];
            new_line.push_str(before);

            if let Some(value) = v {
                new_line.push_str(value);
                new_line.push_str(after);
                new_line.push('\n');
                Ok(ChangeString::Changed(new_line))
            } else {
                Err(DryRunError::VarNotFound(String::from(key)))
            }
        }
        None => Ok(ChangeString::Unchanged),
    }
}
// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file<'a, 'b>(
    vars: &'a HashMap<&str, &str>,
    template: &'b SrcFile,
) -> Result<GenFile, DryRunError> {
    let gen = GenFile::new();
    let maybe_infile: Result<File, Error> = template.open();
    let infile = maybe_infile
        .map_err(|e|DryRunError::FileReadError(e.to_string(), template.to_string()))?;
    let reader = BufReader::new(infile);
    let mut tmpfile: &File = gen.open();
    for maybe_line in reader.lines() {
        let line: String = maybe_line.unwrap();
        match replace_line(vars, line.clone()) {
            Ok(replaced_line) => match replaced_line {
                ChangeString::Changed(new_line) => {
                    writeln!(tmpfile, "{}", new_line).expect("write failed");
                }
                ChangeString::Unchanged => {
                    trace!("no vars in line {:?}", line);
                    writeln!(tmpfile, "{}", line).expect("Cannot write to tmp file");
                }
            },
            Err(e) => return Err(e),
        }
    }
    Ok(gen)
}
