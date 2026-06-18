use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub repos: Vec<RepoConfig>,
}

#[derive(Deserialize)]
pub struct Settings {
    pub poll_interval_secs: u64,
    pub log_poll_interval_secs: u64,
}

#[derive(Deserialize)]
pub struct RepoConfig {
    pub name: String,
    pub provider: String,
    pub owner: String,
    pub repo: String,
    pub token: String,
}

pub fn load_config() -> Result<Config, String> {
    let path = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".config/lazypipe/config.toml");

    let content = std::fs::read_to_string(&path)
        .map_err(|_| format!("Config not found: {}", path.display()))?;

    toml::from_str(&content)
        .map_err(|e| format!("Invalid config: {}", e))
}
