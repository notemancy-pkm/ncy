// src/commands/set.rs
use crate::utils::read_config;
use anyhow::{Context, Result, anyhow};
use serde_json::Value as JsonValue;
use serde_yaml;
use std::env;
use std::fs;
use std::path::Path;

pub fn execute(vault_name: &str) -> Result<()> {
    // Read the NOTEMANCY_CONF_DIR environment variable
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .context("NOTEMANCY_CONF_DIR environment variable is not set")?;

    // Get the config file path
    let config_file_path = Path::new(&conf_dir).join("config.yaml");

    // Read and parse the config using our existing read_config function
    let config = read_config()?;

    // Validate that the specified vault exists
    validate_vault_exists(&config, vault_name)?;

    // Read the original YAML content to preserve formatting and comments
    let yaml_content =
        fs::read_to_string(&config_file_path).context("Failed to read configuration file")?;

    let mut yaml_value = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content)
        .context("Failed to parse YAML configuration")?;

    // Update the default_vault setting
    if let serde_yaml::Value::Mapping(ref mut mapping) = yaml_value {
        mapping.insert(
            serde_yaml::Value::String("default_vault".to_string()),
            serde_yaml::Value::String(vault_name.to_string()),
        );
    }

    // Write the updated config back to the file
    let updated_yaml =
        serde_yaml::to_string(&yaml_value).context("Failed to serialize YAML configuration")?;

    fs::write(&config_file_path, updated_yaml).context("Failed to write updated configuration")?;

    println!("Default vault set to '{}'", vault_name);
    Ok(())
}

fn validate_vault_exists(config: &JsonValue, vault_name: &str) -> Result<()> {
    // Check if vaults key exists and is an array
    let vaults = match config.get("vaults") {
        Some(v) if v.is_array() => v.as_array().unwrap(),
        _ => {
            return Err(anyhow!(
                "No 'vaults' section found in configuration. Please update your config.yaml to include a vaults section."
            ));
        }
    };

    // Check if the specified vault exists in the vaults list
    for vault in vaults {
        if let Some(name) = vault.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str == vault_name {
                    return Ok(());
                }
            }
        }
    }

    // If we get here, the vault wasn't found
    Err(anyhow!(
        "Vault '{}' not found in configuration. Please add it to your config.yaml file first.",
        vault_name
    ))
}
