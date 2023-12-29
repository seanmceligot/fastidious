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
    fn extract_range<'a>(s: &'a str, r: Range<usize>) -> &'a str {
        &s[r.start..r.end]
    }
    let s1 = "a@@foo@@";
    let s2 = "@@foo@@a";
    let s3 = "@@foo@@";
    match match_line(s1) {
        Some((r, _outer)) => {
            assert_eq!(r.start, 3);
            assert_eq!(r.end, 6);
            assert_eq!(extract_range(&s1, r), "foo");
        }
        None => panic!("expected Template"),
    }
    match match_line(s2) {
        Some((r, outer)) => {
            assert_eq!(r.start, 2);
            assert_eq!(r.end, 5);
            assert_eq!(extract_range(s2, r), "foo");
            assert_eq!(extract_range(s2, outer), "@@foo@@");
        }
        None => panic!("expected Template"),
    }
    match match_line(s3) {
        Some((r, _outer)) => {
            assert_eq!(extract_range(s3, r), "foo");
        }
        None => panic!("expected Template"),
    }
}
fn match_line(line: &str) -> Option<(Range<usize>, Range<usize>)> {
    let left_delim = "@@";
    let left_delim_len = left_delim.len();
    let right_delim = "@@";
    let right_delim_len = right_delim.len();
    line.find(left_delim).and_then(|start_of_left_delim| {
        line[start_of_left_delim + left_delim_len..]
            .find(right_delim)
            .map(|start_of_right_delim| {
                (
                    Range {
                        start: start_of_left_delim + left_delim_len,
                        end: start_of_right_delim + start_of_left_delim + left_delim_len,
                    },
                    Range {
                        start: start_of_left_delim,
                        end: (start_of_right_delim
                            + start_of_left_delim
                            + right_delim_len
                            + left_delim_len),
                    },
                )
            })
    })
}
pub enum ChangeString {
    Changed(String),
    Unchanged,
}
pub fn replace_line2(vars: &Vars, line: &str) -> Result<String, ApplyError> {
    match match_line(line) {
        Some((inner, outer)) => {
            debug!("{}", line,);
            let key = &line[inner.start..inner.end];
            trace!("key {}", key);
            let mut new_line: String = String::new();
            let v = vars.get(key);
            trace!("vars {:?}", vars);
            trace!("val {:?}", v);
            trace!("line {:?}", line);
            trace!("inner {} {}", inner.start, inner.end);
            trace!("outer {} {}", outer.start, outer.end);
            let before: &str = &line[..outer.start];
            let after: &str = &line[outer.end..];
            trace!("before {}", before);
            new_line.push_str(before);

            if let Some(value) = v {
                new_line.push_str(value);
                new_line.push_str(after);
                new_line.push('\n');
                trace!("value {}", value);
                trace!("after {}", after);
                Ok(new_line)
            } else {
                Err(ApplyError::VarNotFound(String::from(key)))
            }
        }
        None => Ok(line.to_owned()),
    }
}
pub fn replace_line(vars: &Vars, line: &str) -> Result<ChangeString, ApplyError> {
    match match_line(line) {
        Some((inner, outer)) => {
            debug!("{}", line,);
            let key = &line[inner.start..inner.end];
            trace!("key {}", key);
            let mut new_line: String = String::new();
            let v = vars.get(key);
            trace!("vars {:?}", vars);
            trace!("val {:?}", v);
            trace!("line {:?}", line);
            trace!("inner {} {}", inner.start, inner.end);
            trace!("outer {} {}", outer.start, outer.end);
            let before: &str = &line[..outer.start];
            let after: &str = &line[outer.end..];
            trace!("before {}", before);
            new_line.push_str(before);

            if let Some(value) = v {
                new_line.push_str(value);
                new_line.push_str(after);
                new_line.push('\n');
                trace!("value {}", value);
                trace!("after {}", after);
                Ok(ChangeString::Changed(new_line))
            } else {
                Err(ApplyError::VarNotFound(String::from(key)))
            }
        }
        None => Ok(ChangeString::Unchanged),
    }
}
// creates the tmp file for comparing to the dest file
pub fn generate_recommended_file(vars: Vars, template: &SrcFile) -> Result<GenFile, ApplyError> {
    let gen = GenFile::new()?;
    let maybe_infile = template.open();
    let infile = maybe_infile?;
    let reader = BufReader::new(infile.file());
    let mut tmpfile = gen.open()?;
    for maybe_line in reader.lines() {
        let line: String = maybe_line.unwrap();
        match replace_line(&vars, &line) {
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
