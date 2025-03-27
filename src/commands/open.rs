use crate::utils::read_config;
use anyhow::{Context, Result, anyhow};
use notemancy_core::notes::utils::{get_title, list_all_notes_alt};
use nucleo_picker::{Picker, render::StrRenderer};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

// pub fn execute() -> Result<()> {
//     execute_with_options(false)
// }

pub fn execute_with_options(use_external: bool) -> Result<()> {
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

    // Choose picker based on use_external flag
    if use_external {
        // Use fzf for external mode
        select_note_with_fzf(&note_titles, &title_to_path_map)
    } else {
        // Use nucleo_picker for regular mode
        select_note_with_nucleo(&note_titles, &title_to_path_map)
    }
}

fn select_note_with_nucleo(
    note_titles: &[String],
    title_to_path_map: &std::collections::HashMap<String, String>,
) -> Result<()> {
    // Use nucleo_picker to let the user select a note
    let mut picker = Picker::new(StrRenderer);
    let injector = picker.injector();

    // Clone the titles to avoid lifetime issues with the picker
    for title in note_titles {
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

fn select_note_with_fzf(
    note_titles: &[String],
    title_to_path_map: &std::collections::HashMap<String, String>,
) -> Result<()> {
    // Create a temporary file to store titles for fzf
    let temp_dir = env::temp_dir();
    let temp_file_path = temp_dir.join("ncy_titles.txt");

    // Write titles to the temporary file
    let mut titles_str = String::new();
    for title in note_titles {
        titles_str.push_str(title);
        titles_str.push('\n');
    }
    fs::write(&temp_file_path, &titles_str).context("Failed to write titles to temporary file")?;

    // Set up fzf command with appropriate options
    let mut fzf_cmd = Command::new("fzf")
        .arg("--no-mouse")
        .arg("--height=40%")
        .arg("--layout=reverse")
        .arg("--border")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // Suppress stderr
        .spawn()
        .context("Failed to spawn fzf process. Is fzf installed?")?;

    // Feed the titles to fzf
    if let Some(mut stdin) = fzf_cmd.stdin.take() {
        stdin
            .write_all(titles_str.as_bytes())
            .context("Failed to write to fzf stdin")?;
    }

    // Get the selected title
    let output = fzf_cmd
        .wait_with_output()
        .context("Failed to get output from fzf")?;

    // Clean up the temporary file
    let _ = fs::remove_file(&temp_file_path);

    if !output.status.success() {
        // User cancelled (ESC, Ctrl+C, etc.)
        return Err(anyhow!("No note selected"));
    }

    let selected_title = String::from_utf8(output.stdout)
        .context("Failed to parse fzf output")?
        .trim()
        .to_string();

    if selected_title.is_empty() {
        return Err(anyhow!("No note selected"));
    }

    // Get the file path for the selected note
    let file_path = title_to_path_map.get(&selected_title).context(format!(
        "Could not find file path for note: {}",
        selected_title
    ))?;

    // Print only the absolute path to stdout
    println!("{}", file_path);

    Ok(())
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
