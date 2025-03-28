use anyhow::{Context, Result, anyhow};
use serde_yaml;
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

    // Now we need to read the config file to check vault directories
    let yaml_content = fs::read_to_string(&config_file_path)
        .context("Failed to read configuration file after editing")?;

    // Parse YAML content
    let yaml_value = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content)
        .context("Failed to parse YAML configuration")?;

    // Check if vaults section exists
    let vaults = match &yaml_value {
        serde_yaml::Value::Mapping(map) => {
            if let Some(vaults_value) = map.get(&serde_yaml::Value::String("vaults".to_string())) {
                if let serde_yaml::Value::Sequence(vaults_seq) = vaults_value {
                    Some(vaults_seq)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(vaults) = vaults {
        for vault in vaults {
            if let serde_yaml::Value::Mapping(vault_map) = vault {
                // Get the vault directory
                let vault_dir = vault_map
                    .get(&serde_yaml::Value::String("vault_directory".to_string()))
                    .and_then(|v| {
                        if let serde_yaml::Value::String(dir) = v {
                            Some(dir.as_str())
                        } else {
                            None
                        }
                    });

                // Get the vault name for messaging
                let vault_name = vault_map
                    .get(&serde_yaml::Value::String("name".to_string()))
                    .and_then(|v| {
                        if let serde_yaml::Value::String(name) = v {
                            Some(name.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("unnamed vault");

                if let Some(dir) = vault_dir {
                    let vault_path = Path::new(dir);
                    if !vault_path.exists() {
                        fs::create_dir_all(vault_path)
                            .context(format!("Failed to create vault directory: {}", dir))?;
                        println!("Created vault directory for '{}': {}", vault_name, dir);
                    }

                    // Create journal folder inside vault
                    let journal_path = vault_path.join("journal");
                    if !journal_path.exists() {
                        fs::create_dir_all(&journal_path).context(format!(
                            "Failed to create journal directory: {}",
                            journal_path.display()
                        ))?;
                        println!(
                            "Created journal directory for '{}': {}",
                            vault_name,
                            journal_path.display()
                        );
                    }

                    let workspaces_path = vault_path.join("workspaces");
                    if !workspaces_path.exists() {
                        fs::create_dir_all(&workspaces_path).context(format!(
                            "Failed to create workspaces directory: {}",
                            workspaces_path.display()
                        ))?;
                        println!(
                            "Created workspaces directory for '{}': {}",
                            vault_name,
                            workspaces_path.display()
                        );
                    }
                }
            }
        }
    }

    println!("Configuration completed successfully!");
    Ok(())
}
