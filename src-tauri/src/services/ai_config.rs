use crate::engine::ai::AiConfig;
use crate::model::AiProvider;
use std::fs;
use std::path::PathBuf;

fn config_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or("Cannot determine app data directory")?;
    let dir = base.join("iconstudio");
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create config dir: {}", e))?;
    Ok(dir)
}

fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("ai_config.json"))
}

fn obfuscate(input: &str) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, input.as_bytes())
}

fn deobfuscate(input: &str) -> String {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, input)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredConfig {
    provider: String,
    api_key: String,
    model: String,
    endpoint: Option<String>,
    timeout_secs: u64,
}

pub fn save_ai_config(config: &AiConfig) -> Result<(), String> {
    let provider_str = match config.provider {
        AiProvider::OpenAi => "openAi",
        AiProvider::Recraft => "recraft",
        AiProvider::Custom => "custom",
        AiProvider::Ollama => "ollama",
    };

    let stored = StoredConfig {
        provider: provider_str.to_string(),
        api_key: obfuscate(&config.api_key),
        model: config.model.clone(),
        endpoint: config.endpoint.clone(),
        timeout_secs: config.timeout_secs,
    };

    let path = config_path()?;
    let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| format!("Cannot write AI config: {}", e))?;
    Ok(())
}

pub fn load_ai_config() -> Result<AiConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AiConfig::default());
    }

    let json = fs::read_to_string(path).map_err(|e| format!("Cannot read AI config: {}", e))?;
    let stored: StoredConfig =
        serde_json::from_str(&json).map_err(|e| format!("Cannot parse AI config: {}", e))?;

    let provider = match stored.provider.as_str() {
        "openAi" => AiProvider::OpenAi,
        "recraft" => AiProvider::Recraft,
        "custom" => AiProvider::Custom,
        "ollama" => AiProvider::Ollama,
        _ => AiProvider::OpenAi,
    };

    Ok(AiConfig {
        provider,
        api_key: deobfuscate(&stored.api_key),
        model: stored.model,
        endpoint: stored.endpoint,
        timeout_secs: stored.timeout_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_deobfuscate() {
        let original = "sk-test-api-key-12345";
        let obfuscated = obfuscate(original);
        assert_ne!(obfuscated, original);
        assert_eq!(deobfuscate(&obfuscated), original);
    }

    #[test]
    fn test_obfuscate_empty() {
        assert_eq!(deobfuscate(&obfuscate("")), "");
    }
}
