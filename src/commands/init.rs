use anyhow::{Context, Result, anyhow};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute() -> Result<()> {
    // Read the NOTEMANCY_CONF_DIR environment variable
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .context("NOTEMANCY_CONF_DIR environment variable is not set")?;

    // Create the config directory if it doesn't exist
    let conf_path = Path::new(&conf_dir);
    if !conf_path.exists() {
        fs::create_dir_all(conf_path).context("Failed to create configuration directory")?;
        println!("Created configuration directory: {}", conf_dir);
    }

    // Create the config file path
    let config_file_path = conf_path.join("config.yaml");

    // If the config file doesn't exist, create an empty one
    if !config_file_path.exists() {
        fs::write(&config_file_path, "").context("Failed to create empty configuration file")?;

        println!("Created empty configuration file");
    }

    // Open the config file in the default editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    println!("Opening configuration file with {}", editor);

    let status = Command::new(&editor)
        .arg(&config_file_path)
        .status()
        .context(format!(
            "Failed to open editor '{}' for config file",
            editor
        ))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status"));
    }

    println!("Configuration completed successfully!");
    Ok(())
}
