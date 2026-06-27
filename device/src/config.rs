use anyhow::Context;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: String,
    pub name: String,
    pub secret_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub feeds: Vec<Feed>,
    pub poll_interval_minutes: u64,
    pub hostname: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublicFeed {
    pub id: String,
    pub name: String,
}

impl Config {
    pub fn default() -> Self {
        Config {
            feeds: vec![],
            poll_interval_minutes: 60,
            hostname: "corkboard.local".to_string(),
        }
    }

    pub fn public_feeds(&self) -> Vec<PublicFeed> {
        self.feeds
            .iter()
            .map(|f| PublicFeed {
                id: f.id.clone(),
                name: f.name.clone(),
            })
            .collect()
    }

    pub fn load(path: &std::path::Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                tracing::warn!("config.json could not be read; using defaults");
                return Self::default();
            }
        };
        match serde_json::from_str(&data) {
            Ok(cfg) => cfg,
            Err(_) => {
                tracing::warn!("config.json failed to parse; using defaults");
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create dir {:?}", parent))?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data).with_context(|| format!("write {:?}", path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_feeds_strips_secret_url() {
        let cfg = Config {
            feeds: vec![Feed {
                id: "feed-1".to_string(),
                name: "My Feed".to_string(),
                secret_url: "https://secret.example.com/token123".to_string(),
            }],
            poll_interval_minutes: 30,
            hostname: "test.local".to_string(),
        };

        let public = cfg.public_feeds();
        assert_eq!(public.len(), 1);
        assert_eq!(public[0].name, "My Feed");

        let serialized = serde_json::to_string(&public).unwrap();
        assert!(!serialized.contains("secret.example.com"));
        assert!(!serialized.contains("token123"));
        assert!(!serialized.contains("secretUrl"));
    }

    #[test]
    fn save_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        let cfg = Config {
            feeds: vec![Feed {
                id: "f1".to_string(),
                name: "Feed One".to_string(),
                secret_url: "https://example.com/secret".to_string(),
            }],
            poll_interval_minutes: 15,
            hostname: "mydevice.local".to_string(),
        };

        cfg.save(&path).unwrap();
        let loaded = Config::load(&path);

        assert_eq!(loaded.feeds.len(), 1);
        assert_eq!(loaded.feeds[0].id, "f1");
        assert_eq!(loaded.feeds[0].name, "Feed One");
        assert_eq!(loaded.feeds[0].secret_url, "https://example.com/secret");
        assert_eq!(loaded.poll_interval_minutes, 15);
        assert_eq!(loaded.hostname, "mydevice.local");
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let cfg = Config::load(&path);
        assert_eq!(cfg.feeds.len(), 0);
        assert_eq!(cfg.poll_interval_minutes, 60);
    }

    // M3: corrupt config silently resets (now with warn, no panic, no URL leak)
    #[test]
    fn load_corrupt_config_returns_default_without_panic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, b"{ this is not valid json !!").unwrap();
        let cfg = Config::load(&path);
        // Must return defaults, not panic
        assert_eq!(cfg.feeds.len(), 0);
        assert_eq!(cfg.poll_interval_minutes, 60);
        assert_eq!(cfg.hostname, "corkboard.local");
    }
}
