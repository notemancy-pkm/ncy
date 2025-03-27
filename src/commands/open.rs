use crate::utils::read_config;
use anyhow::{Context, Result, anyhow};
use notemancy_core::notes::utils::{get_title, list_all_notes_alt};
use nucleo_picker::{Picker, render::StrRenderer};
use std::env;
use std::path::Path;
use std::process::Command;

pub fn execute() -> Result<()> {
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

    // Get all markdown notes in the vault
    let all_notes = list_all_notes_alt(vault_path, false)?;

    if all_notes.is_empty() {
        return Err(anyhow!(
            "No markdown notes found in vault: {}",
            default_vault
        ));
    }

    // Get the title for each note
    let mut note_titles = Vec::new();
    let mut title_to_path_map = std::collections::HashMap::new();

    for note_path in all_notes {
        let path = Path::new(&note_path);
        let title = get_title(path)?;
        note_titles.push(title.clone());
        title_to_path_map.insert(title, note_path);
    }

    // Use nucleo_picker to let the user select a note
    let mut picker = Picker::new(StrRenderer);
    let injector = picker.injector();

    // Clone the titles to avoid lifetime issues with the picker
    for title in &note_titles {
        injector.push(title.clone());
    }

    // Open interactive prompt
    match picker.pick()? {
        Some(selected_title) => {
            // Get the file path for the selected note
            let file_path = title_to_path_map.get(selected_title).context(format!(
                "Could not find file path for note: {}",
                selected_title
            ))?;

            // Open the note in the default editor
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            println!("Opening note: {} with {}", selected_title, editor);

            let status = Command::new(&editor)
                .arg(file_path)
                .status()
                .context(format!("Failed to open editor '{}' for note", editor))?;

            if !status.success() {
                return Err(anyhow!("Editor exited with non-zero status"));
            }

            Ok(())
        }
        None => Err(anyhow!("No note selected")),
    }
}

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
