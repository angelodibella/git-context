use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Context {
    pub path: PathBuf,
    pub managed_files: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub active_context: String,
    pub contexts: HashMap<String, Context>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(".contexts")
            .context("Failed to read contexts file. Have you run 'init'?")?;
        let parsed = toml::from_str(&content).context("Failed to parse contexts file. There might be semantic errors inside it, or it might be corrupted.")?;
        Ok(parsed)
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config object")?;
        fs::write(".contexts", content).context("Failed to write to contexts file")?;
        Ok(())
    }
}
