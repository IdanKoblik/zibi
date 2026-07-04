use serde::Deserialize;
use serde::Serialize;
use std::{fs, path::PathBuf};
use tracing::error;
use tracing::info;
use zibi_core::hand::Hand;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub core: CoreConfig,
}

#[derive(Deserialize, Serialize)]
pub struct CoreConfig {
    pub move_threshold: i16, /* Ms */
    pub dominant_hand: Hand,
    pub camera: PathBuf,
}

impl Config {
    fn default() -> Self {
        Self {
            core: CoreConfig {
                move_threshold: 150,
                dominant_hand: Hand::Right,
                camera: std::path::PathBuf::from("/dev/video0"),
            },
        }
    }

    pub fn path() -> PathBuf {
        let base = match dirs::home_dir() {
            Some(dir) => dir,
            None => {
                error!("no home dir");
                panic!("no home dir");
            }
        };

        let mut p = base.join(".config");
        p.push("zibi");
        p.push("config.toml");
        p
    }

    pub fn load() -> Self {
        let path = Self::path();

        info!("Config path: {}", path.to_string_lossy());
        if !path.exists() {
            info!("Config not found, creating default one");
            let cfg = Self::default();
            if let Err(e) = cfg.save() {
                error!("failed to write default config, {e}");
                panic!("failed to write default config");
            }

            return cfg;
        }

        let content = fs::read_to_string(&path).unwrap_or_else(|_| {
            error!("cannot read config at {:?},", path);
            panic!("cannot read config at {:?}", path);
        });

        toml::from_str(&content).unwrap_or_else(|_| Self::default())
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = toml::to_string_pretty(self).unwrap_or_else(|_| {
            error!("serialize failed");
            panic!("serialize failed");
        });

        fs::write(path, data)?;
        Ok(())
    }
}
