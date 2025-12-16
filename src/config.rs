use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Context {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub active_context: String,
    pub managed_files: Vec<PathBuf>,
    pub contexts: HashMap<String, Context>,
}
