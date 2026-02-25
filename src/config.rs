use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Prod,
    Demo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredConfig {
    pub api_key: Option<String>,
    pub api_secret_path: Option<String>,
    pub environment: Option<Environment>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub environment: Environment,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
}

impl RuntimeConfig {
    pub fn rest_base_url(&self) -> &'static str {
        match self.environment {
            Environment::Prod => "https://api.elections.kalshi.com/trade-api/v2",
            Environment::Demo => "https://demo-api.kalshi.co/trade-api/v2",
        }
    }

    pub fn ws_url(&self) -> &'static str {
        match self.environment {
            Environment::Prod => "wss://api.elections.kalshi.com/trade-api/ws/v2",
            Environment::Demo => "wss://demo-api.kalshi.co/trade-api/ws/v2",
        }
    }
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().or_else(|| dirs::home_dir().map(|h| h.join(".config"))).unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join("kalshi-cli").join("config.json"))
}

pub fn ensure_auth(runtime: &RuntimeConfig) -> anyhow::Result<()> {
    if runtime.api_key.is_none() || runtime.api_secret.is_none() {
        anyhow::bail!(
            "This command requires auth. Configure credentials via `kal config setup`, env vars, or --api-key/--api-secret"
        );
    }
    Ok(())
}

pub fn load_stored_config() -> anyhow::Result<StoredConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(StoredConfig::default());
    }

    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str::<StoredConfig>(&content)?;
    Ok(config)
}

pub fn save_config(config: &StoredConfig) -> anyhow::Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_string_pretty(config)?;
    let mut file = fs::File::create(&path)?;
    file.write_all(body.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn delete_config() -> anyhow::Result<()> {
    let path = config_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn resolve_runtime_config(
    cli_environment: Option<Environment>,
    cli_api_key: Option<String>,
    cli_api_secret: Option<String>,
) -> anyhow::Result<RuntimeConfig> {
    let file_cfg = load_stored_config()?;

    let environment = cli_environment
        .or_else(|| std::env::var("KALSHI_ENV").ok().and_then(parse_env))
        .or(file_cfg.environment)
        .unwrap_or(Environment::Prod);

    let api_key = cli_api_key
        .or_else(|| std::env::var("KALSHI_API_KEY").ok())
        .or(file_cfg.api_key);

    let api_secret = match cli_api_secret {
        Some(secret_or_path) => Some(resolve_secret(&secret_or_path)?),
        None => {
            if let Ok(raw) = std::env::var("KALSHI_API_SECRET") {
                Some(resolve_secret(&raw)?)
            } else if let Some(path) = file_cfg.api_secret_path {
                Some(read_secret_file(Path::new(&path))?)
            } else {
                None
            }
        }
    };

    Ok(RuntimeConfig {
        environment,
        api_key,
        api_secret,
    })
}

pub fn resolve_secret(input: &str) -> anyhow::Result<String> {
    let path = Path::new(input);
    if path.exists() {
        return read_secret_file(path);
    }
    Ok(input.to_string())
}

fn read_secret_file(path: &Path) -> anyhow::Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

fn parse_env(value: String) -> Option<Environment> {
    match value.to_lowercase().as_str() {
        "prod" => Some(Environment::Prod),
        "demo" => Some(Environment::Demo),
        _ => None,
    }
}
