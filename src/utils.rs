use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use serde_yaml;
use std::env;
use std::fs;
use std::path::Path;

// Function to read and parse the configuration file
pub fn read_config() -> Result<JsonValue> {
    // Read the NOTEMANCY_CONF_DIR environment variable
    let conf_dir = env::var("NOTEMANCY_CONF_DIR")
        .context("NOTEMANCY_CONF_DIR environment variable is not set")?;

    // Construct the path to the config file
    let config_path = Path::new(&conf_dir).join("config.yaml");

    // Check if the file exists
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found. Run 'ncy init' first."
        ));
    }

    // Read the file content
    let yaml_content =
        fs::read_to_string(&config_path).context("Failed to read configuration file")?;

    // Parse YAML content
    let yaml_value = serde_yaml::from_str::<serde_yaml::Value>(&yaml_content)
        .context("Failed to parse YAML configuration")?;

    // Convert YAML to JSON
    let json_value = yaml_to_json(yaml_value);

    Ok(json_value)
}

// Function to convert serde_yaml::Value to serde_json::Value
fn yaml_to_json(yaml: serde_yaml::Value) -> JsonValue {
    match yaml {
        serde_yaml::Value::Null => JsonValue::Null,
        serde_yaml::Value::Bool(b) => JsonValue::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                // Use from_f64 and handle non-finite values
                JsonValue::Number(
                    serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
                )
            } else {
                JsonValue::Null // Fallback, should not happen
            }
        }
        serde_yaml::Value::String(s) => JsonValue::String(s),
        serde_yaml::Value::Sequence(seq) => {
            JsonValue::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    json_map.insert(key, yaml_to_json(v));
                } else {
                    // Convert non-string keys to strings (JSON requires string keys)
                    let key = match k {
                        serde_yaml::Value::Null => "null".to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::String(s) => s,
                        // Combined the cases for Sequence and Mapping
                        _ => format!("{:?}", k),
                    };
                    json_map.insert(key, yaml_to_json(v));
                }
            }
            JsonValue::Object(json_map)
        }
    }
}
