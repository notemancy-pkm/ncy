// src/commands/jrnl.rs
use crate::utils::read_config;
use anyhow::{Context, Result, anyhow};
use chrono::Local;
use notemancy_core::notes::crud::{append_to_note, create_note};
use notemancy_core::notes::utils::get_file_path;
use std::env;
use std::path::Path;
use std::process::Command;

pub fn execute(args: &str) -> Result<()> {
    // If args is empty, open today's journal entry
    if args.is_empty() {
        return open_todays_journal();
    }

    // Get configuration
    let config = read_config()?;

    // Get default vault from config
    let default_vault = config
        .get("default_vault")
        .and_then(|v| v.as_str())
        .context("No default vault set. Run 'ncy set <vault-name>' first.")?;

    // Find the vault directory for the default vault
    let vault_directory = find_vault_directory(&config, default_vault)?;
    let vault_path = Path::new(&vault_directory);

    // Get today's date in MM-DD-YYYY format
    let today = Local::now();
    let date_str = today.format("%m-%d-%Y").to_string();

    // Set up the journal path inside the vault
    let journal_project = "journal";

    // Format the text to append with the separator
    let text_to_append = format!("\n\n--\n{}", args);

    // Check if today's journal entry exists
    let note_path = match get_file_path(&date_str, vault_path) {
        Ok(_) => {
            // Today's journal exists, append to it
            append_to_note(&date_str, vault_path, &text_to_append)?;
            println!("Added entry to today's journal ({}).", date_str);
            None
        }
        Err(_) => {
            // Today's journal doesn't exist, create it
            let new_note_path = create_note(&date_str, vault_path, journal_project)?;

            // Now append the content (since create_note only creates with frontmatter)
            append_to_note(&date_str, vault_path, &text_to_append)?;

            println!("Created new journal entry for today ({}).", date_str);
            Some(new_note_path)
        }
    };

    if let Some(path) = note_path {
        println!("Journal entry created at: {}", path.display());
    }

    Ok(())
}

// Function to open today's journal entry
fn open_todays_journal() -> Result<()> {
    // Get configuration
    let config = read_config()?;

    // Get default vault from config
    let default_vault = config
        .get("default_vault")
        .and_then(|v| v.as_str())
        .context("No default vault set. Run 'ncy set <vault-name>' first.")?;

    // Find the vault directory for the default vault
    let vault_directory = find_vault_directory(&config, default_vault)?;
    let vault_path = Path::new(&vault_directory);

    // Get today's date in MM-DD-YYYY format
    let today = Local::now();
    let date_str = today.format("%m-%d-%Y").to_string();

    // Set up the journal path inside the vault
    let journal_project = "journal";

    // Check if today's journal entry exists
    let note_path = match get_file_path(&date_str, vault_path) {
        Ok(path) => {
            // Today's journal exists, open it
            println!("Opening today's journal entry ({}).", date_str);
            path
        }
        Err(_) => {
            // Today's journal doesn't exist, create it
            let new_note_path = create_note(&date_str, vault_path, journal_project)?;
            println!("Created new journal entry for today ({}).", date_str);
            new_note_path.to_string_lossy().to_string()
        }
    };

    // Open the note in the default editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    println!("Opening journal with {}", editor);

    let status = Command::new(&editor)
        .arg(&note_path)
        .status()
        .context(format!("Failed to open editor '{}' for journal", editor))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status"));
    }

    Ok(())
}

// Reusing the function from other commands to find vault directory
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
