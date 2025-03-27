// src/commands/new.rs
use crate::utils::read_config;
use anyhow::{Context, Result, anyhow};
use notemancy_core::notes::crud::create_note;
use std::env;
use std::path::Path;
use std::process::Command;

pub fn execute(args: &str) -> Result<()> {
    // Parse the arguments: "title @ project/path +vault"
    let (title, project, vault) = parse_arguments(args)?;

    // Get configuration
    let config = read_config()?;

    // Determine which vault to use
    let vault_name = if let Some(v) = vault {
        // Use explicitly specified vault
        validate_vault_exists(&config, &v)?;
        v
    } else {
        // Use default vault from config
        config
            .get("default_vault")
            .and_then(|v| v.as_str())
            .context("No default vault set. Run 'ncy set <vault-name>' first or specify a vault with '+vault'.")?
            .to_string()
    };

    // Find the vault directory for the selected vault
    let vault_directory = find_vault_directory(&config, &vault_name)?;
    let vault_path = Path::new(&vault_directory);

    // Create the note
    let note_path = create_note(&title, vault_path, &project).context(format!(
        "Failed to create note '{}' in project '{}'",
        title, project
    ))?;

    println!("Created note: {} in {}", title, note_path.display());

    // Open the note in the default editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    println!("Opening note with {}", editor);

    let status = Command::new(&editor)
        .arg(&note_path)
        .status()
        .context(format!("Failed to open editor '{}' for note", editor))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status"));
    }

    Ok(())
}

/// Parses the command arguments in the format: "title @ project/path +vault"
/// Returns a tuple of (title, project, vault) where vault is an Option<String>
fn parse_arguments(args: &str) -> Result<(String, String, Option<String>)> {
    if args.is_empty() {
        return Err(anyhow!("Note title is required"));
    }

    // Initialize default values
    let mut title = args.to_string();
    let mut project = String::new();
    let mut vault = None;

    // Check for vault specification (after '+')
    if let Some(plus_pos) = args.rfind('+') {
        vault = Some(args[plus_pos + 1..].trim().to_string());
        title = args[..plus_pos].trim().to_string();
    }

    // Check for project path specification (after '@')
    if let Some(at_pos) = title.rfind('@') {
        project = title[at_pos + 1..].trim().to_string();
        title = title[..at_pos].trim().to_string();

        // If we found a project path and there's also a vault, we need to adjust
        if let Some(ref mut v) = vault {
            // Check if the vault was accidentally included in the project
            if let Some(plus_in_project) = project.rfind('+') {
                *v = project[plus_in_project + 1..].trim().to_string();
                project = project[..plus_in_project].trim().to_string();
            }
        }
    }

    // Ensure title is not empty after parsing
    if title.is_empty() {
        return Err(anyhow!("Note title cannot be empty"));
    }

    Ok((title, project, vault))
}

// Reusing the function from set.rs for validation
fn validate_vault_exists(config: &serde_json::Value, vault_name: &str) -> Result<()> {
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

// Reusing the function from open.rs to find vault directory
fn find_vault_directory(config: &serde_json::Value, vault_name: &str) -> Result<String> {
    // Get the vaults array from config
    let vaults = config
        .get("vaults")
        .and_then(|v| v.as_array())
        .context("No vaults defined in configuration")?;

    // Find the vault with the specified name
    for vault in vaults {
        if let Some(name) = vault.get("name").and_then(|n| n.as_str()) {
            if name == vault_name {
                if let Some(dir) = vault.get("vault_directory").and_then(|d| d.as_str()) {
                    return Ok(dir.to_string());
                }
            }
        }
    }

    Err(anyhow!(
        "Could not find directory for vault: {}",
        vault_name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arguments_with_title_only() {
        let args = "My New Note";
        let (title, project, vault) = parse_arguments(args).unwrap();
        assert_eq!(title, "My New Note");
        assert_eq!(project, "");
        assert_eq!(vault, None);
    }

    #[test]
    fn test_parse_arguments_with_title_and_project() {
        let args = "My New Note @ projects/research";
        let (title, project, vault) = parse_arguments(args).unwrap();
        assert_eq!(title, "My New Note");
        assert_eq!(project, "projects/research");
        assert_eq!(vault, None);
    }

    #[test]
    fn test_parse_arguments_with_title_and_vault() {
        let args = "My New Note +personal";
        let (title, project, vault) = parse_arguments(args).unwrap();
        assert_eq!(title, "My New Note");
        assert_eq!(project, "");
        assert_eq!(vault, Some("personal".to_string()));
    }

    #[test]
    fn test_parse_arguments_with_title_project_and_vault() {
        let args = "My New Note @ projects/research +personal";
        let (title, project, vault) = parse_arguments(args).unwrap();
        assert_eq!(title, "My New Note");
        assert_eq!(project, "projects/research");
        assert_eq!(vault, Some("personal".to_string()));
    }

    #[test]
    fn test_parse_arguments_with_empty_title() {
        let args = "";
        let result = parse_arguments(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_arguments_with_multiple_at_symbols() {
        let args = "Email @user@example.com @ projects/email";
        let (title, project, vault) = parse_arguments(args).unwrap();
        assert_eq!(title, "Email @user@example.com");
        assert_eq!(project, "projects/email");
        assert_eq!(vault, None);
    }

    #[test]
    fn test_parse_arguments_with_spaces() {
        let args = "  My Note  @  project/path  +  vault  ";
        let (title, project, vault) = parse_arguments(args).unwrap();
        assert_eq!(title, "My Note");
        assert_eq!(project, "project/path");
        assert_eq!(vault, Some("vault".to_string()));
    }
}
