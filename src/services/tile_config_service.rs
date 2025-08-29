// services/tile_config_service.rs
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, Clone)]
pub struct TileConfig {
    pub lst_tile_url: String,
    pub anomaly_tile_url: String,
    pub absolute_anomaly_tile_url: String,
}

#[derive(Clone)]
pub struct TileConfigService {
    pub cache: Arc<RwLock<HashMap<i32, TileConfig>>>, // year -> config
    base_path: String,                                // folder path where tile configs are stored
}

impl TileConfigService {
    /// Create a new service with a given base path
    pub fn new(base_path: &str) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            base_path: base_path.to_string(),
        }
    }

    /// Load tile config for a specific year from disk
    pub async fn load_config_for_year(&self, year: i32) -> Result<()> {
        // Construct file path
        let file_path = format!("{}/tiles/tile_config_{}.json", self.base_path, year);

        // Debug: print the path to make sure it's correct
        println!("Loading tile config from: {}", file_path);

        // Read the file
        let config_data = fs::read_to_string(&file_path).await?;
        let config: TileConfig = serde_json::from_str(&config_data)?;

        // Insert into cache
        let mut write_guard = self.cache.write().await;
        write_guard.insert(year, config);
        Ok(())
    }

    /// Get tile config for a specific year (tries cache first, then disk)
    pub async fn get_config_for_year(&self, year: i32) -> Option<TileConfig> {
        // Try cache first
        let read_guard = self.cache.read().await;
        if let Some(cfg) = read_guard.get(&year) {
            return Some(cfg.clone());
        }
        drop(read_guard); // release read lock before loading

        // Try loading from disk
        if self.load_config_for_year(year).await.is_ok() {
            let read_guard = self.cache.read().await;
            read_guard.get(&year).cloned()
        } else {
            None
        }
    }

    /// Get tile URLs for a specific year
    pub async fn get_tile_urls_for_year(&self, year: i32) -> Option<(String, String, String)> {
        self.get_config_for_year(year).await.map(|cfg| {
            (
                cfg.lst_tile_url,
                cfg.anomaly_tile_url,
                cfg.absolute_anomaly_tile_url,
            )
        })
    }

    /// List all loaded years
    pub async fn get_loaded_years(&self) -> Vec<i32> {
        let read_guard = self.cache.read().await;
        read_guard.keys().cloned().collect()
    }
}

// -----------------------
// Example usage
// -----------------------
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set your folder where Python script generates tile JSON configs
    let base_path = "C:/Users/amina/app/climate-backend";

    // Create the service
    let tile_service = TileConfigService::new(base_path);

    // Preload a specific year (optional)
    tile_service.load_config_for_year(2025).await?;

    // Fetch tile URLs for 2025
    if let Some((lst, anomaly, abs_anomaly)) = tile_service.get_tile_urls_for_year(2025).await {
        println!("LST Tile: {}", lst);
        println!("Anomaly Tile: {}", anomaly);
        println!("Absolute Anomaly Tile: {}", abs_anomaly);
    } else {
        println!("Tile config not found for 2025");
    }

    Ok(())
}
