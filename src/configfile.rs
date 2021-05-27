
use config::Config;

use std::{collections::HashMap, path::{PathBuf}};

use crate::applyerr::ApplyError;

pub (crate) fn load_config(c1: &mut Config) -> Result<&mut Config, config::ConfigError> {
    let c2 = c1.merge(config::Environment::with_prefix("NONAME"))?;
    let c3 = c2.merge(config::File::with_name("noname").required(false));
    c3
}

fn get_script_directory(conf: &mut Config) -> PathBuf {
    let script_dir = match conf.get_str("script_dir") {
        Ok(val) => PathBuf::from(val),
        Err(_) => dirs::home_dir().unwrap_or(std::env::current_dir().unwrap()),
    };
    println!("script dir: {:?}", script_dir);
    let script_path = PathBuf::from(script_dir);
    if !script_path.exists() {
        panic!("{:?} does not exist", script_path);
    }
    script_path
}
pub(crate) fn find_scriptlet(conf: &mut Config, name: &str, action: &str) -> PathBuf {
    let filename = format!("{}-{}", name, action);
    debug!("script filename {}", filename);
    let dir = get_script_directory(conf);
    debug!("dir {:?}", dir);
    let path = dir.join(filename);
    trace!("script{:?}", path);
    if !path.exists() {
        println!("create file {:?}", path);
    }
    println!("apply script {:?}", path);
    path
}
pub(crate) fn scriptlet_config(conf: &mut Config, name: &str) -> Result<HashMap<String, String>, ApplyError> {
    let maybe_name_config: HashMap<String, config::Value> = conf.get_table(name).map_err(|e| ApplyError::NameNotFound(e.to_string()))?;
    debug!("maybe_name_config {:#?}", maybe_name_config);
    let mut name_config: HashMap<String, String> = HashMap::new();
    for (k, v) in maybe_name_config {
        name_config.insert(k, v.into_str().unwrap());
    }
    Ok(name_config)
}

