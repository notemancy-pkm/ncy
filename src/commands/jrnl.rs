// src/commands/jrnl.rs
use crate::utils::read_config;
use anyhow::{Context, Result, anyhow};
use chrono::Local;
use notemancy_core::notes::crud::{append_to_note, create_note};
use notemancy_core::notes::utils::get_file_path;
use std::env;
use std::path::Path;
use std::process::Command;

pub fn execute(args: &str, external: bool) -> Result<()> {
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

    // Variable to hold the note path
    let note_path;

    // If args is empty, we're just accessing today's journal
    if args.is_empty() {
        // Check if today's journal entry exists
        match get_file_path(&date_str, vault_path) {
            Ok(path) => {
                // Today's journal exists
                if !external {
                    println!("Opening today's journal entry ({}).", date_str);
                }
                note_path = path;
            }
            Err(_) => {
                // Today's journal doesn't exist, create it
                let new_note_path = create_note(&date_str, vault_path, journal_project)?;
                if !external {
                    println!("Created new journal entry for today ({}).", date_str);
                }
                note_path = new_note_path.to_string_lossy().to_string();
            }
        }

        // Only open the editor if we're not in external mode and no args were provided
        if !external {
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
        }
    } else {
        // Adding text to today's journal
        // Format the text to append with the separator
        let text_to_append = format!("\n\n--\n{}", args);

        // Check if today's journal entry exists
        match get_file_path(&date_str, vault_path) {
            Ok(path) => {
                // Today's journal exists, append to it
                append_to_note(&date_str, vault_path, &text_to_append)?;
                println!("Added entry to today's journal ({}).", date_str);
                note_path = path;
            }
            Err(_) => {
                // Today's journal doesn't exist, create it
                let new_note_path = create_note(&date_str, vault_path, journal_project)?;

                // Now append the content (since create_note only creates with frontmatter)
                append_to_note(&date_str, vault_path, &text_to_append)?;

                println!("Created new journal entry for today ({}).", date_str);
                note_path = new_note_path.to_string_lossy().to_string();
            }
        }
    }

    // If external mode is enabled, print the path regardless of whether args were provided
    if external {
        // Just print the path to stdout
        println!("{}", note_path);
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
