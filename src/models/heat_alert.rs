use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct HeatAlert {
    pub city: String,
    pub temperature: f64,
    pub severity: String,
    pub time_ago: String,
}
