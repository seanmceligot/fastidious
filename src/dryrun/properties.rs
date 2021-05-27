extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::path::Path;

pub fn _properties(map: &mut HashMap<String, String>, property_file: String) -> Result<(), Error> {
    let path = Path::new(property_file.as_str());
    println!("open {:?}", path);
    let file = File::open(&path)?;
    let re = Regex::new(r"^(?P<k>[[:alnum:]\._]*)=(?P<v>.*)").unwrap();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        for cap in re.captures_iter(line.unwrap().as_str()) {
            map.insert(
                String::from(cap.name("k").unwrap().as_str()),
                String::from(cap.name("v").unwrap().as_str()),
            );
        }
    }
    println!("map {:?}", map);
    Ok(())
}
