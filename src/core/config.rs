use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEnvelope<T> {
    pub schema_version: u32,
    pub data: T,
}

pub fn load_config<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config '{}': {}", path, e))?;
    ron::from_str::<T>(&contents)
        .map_err(|e| format!("Failed to parse config '{}': {}", path, e))
}

pub fn load_versioned_config<T: DeserializeOwned>(
    path: &str,
    expected_version: u32,
) -> Result<T, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config '{}': {}", path, e))?;
    let envelope: ConfigEnvelope<T> = ron::from_str(&contents)
        .map_err(|e| format!("Failed to parse config '{}': {}", path, e))?;

    if envelope.schema_version > expected_version {
        return Err(format!(
            "Config '{}' has version {} but engine supports up to {}",
            path, envelope.schema_version, expected_version
        ));
    }

    Ok(envelope.data)
}
