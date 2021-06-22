use applyerr::ApplyError;
use files::{GenFile, SrcFile};
use log::trace;
use regex::Match;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::ops::Range;
use std::usize;

use crate::cmd::Vars;

#[test]
fn test_match_line() {
    let s1 = "a@@foo@@";
    let s2 = "@@foo@@a";
    let s3 = "@@foo@@";
    match match_line(s1) {
        Some(r) => {
            assert_eq!(r.start, 3);
            assert_eq!(r.end, 6);
            assert_eq!(s1[r.start..r.end], *"foo");
        }
        None => panic!("expected Template"),
    }
    match match_line(s2) {
        Some(r) => {
            assert_eq!(r.start, 2);
            assert_eq!(r.end, 5);
            assert_eq!(s2[r.start..r.end], *"foo");
        }
        None => panic!("expected Template"),
    }
    match match_line(s3) {
        Some(r) => {
            assert_eq!(s3[r.start..r.end], *"foo");
        }
        None => panic!("expected Template"),
    }
}
fn match_line<'a>(line: &'a str) -> Option<Range<usize>> {
    let start_match = "@@";
    let slen = start_match.len();
    let end_match = "@@";
    match line.find(start_match) {
        Some(match_start) => {
            match line[match_start + slen..].find(end_match) {
                // a@@foo@@
                Some(match_end) => Some(Range {
                    start: match_start + slen,
                    end: match_end + match_start + slen,
                }),
                None => None,
            }
        }
        None => None,
    }
}
pub enum ChangeString {
    Changed(String),
    Unchanged,
}
pub fn replace_line(vars: Vars, line: String) -> Result<ChangeString, ApplyError> {
    match match_line(line.as_str()) {
        Some(range) => {
            debug!(
                "slice {} {} {} {}",
                range.start,
                range.end,
                line,
                line.len()
            );
            let key = &line[range.start..range.end];
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
                Err(ApplyError::VarNotFound(String::from(key)))
            }
        }
        None => Ok(ChangeString::Unchanged),
    }
}
// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file<'a, 'b>(
    vars: Vars,
    template: &'b SrcFile,
) -> Result<GenFile, ApplyError> {
    let gen = GenFile::new()?;
    let maybe_infile = template.open();
    let infile =
        maybe_infile.map_err(|e| ApplyError::FileReadError(format!("{:?} {:?}", template, e)))?;
    let reader = BufReader::new(infile.file());
    let mut tmpfile = gen.open()?;
    for maybe_line in reader.lines() {
        let line: String = maybe_line.unwrap();
        match replace_line(vars.clone(), line.clone()) {
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
