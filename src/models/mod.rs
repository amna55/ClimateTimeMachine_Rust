pub mod heat_alert;
// models/tile_config.rs
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TileConfig {
    pub lst_tile_url: String,
    pub anomaly_tile_url: String,
    pub absolute_anomaly_tile_url: String,
    pub generated_at: String,
    pub target_period: String,
    pub climatology_period: String,
    pub data_source: String,
    pub description: String,
}
