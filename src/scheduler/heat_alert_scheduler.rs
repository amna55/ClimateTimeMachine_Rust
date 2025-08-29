use crate::models::heat_alert::HeatAlert;
use tokio::sync::RwLock;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use crate::services::heat_alert_services::get_top_heat_alerts;

pub struct HeatAlertCache {
    pub top_alerts: RwLock<Vec<HeatAlert>>,
}

impl HeatAlertCache {
    pub fn new() -> Self {
        HeatAlertCache { top_alerts: RwLock::new(vec![]) }
    }
}

pub async fn run_scheduler(cache: Arc<HeatAlertCache>) {
    let mut ticker = interval(Duration::from_secs(6 * 3600)); // 6 hours

    loop {
        ticker.tick().await;

        let top_alerts = get_top_heat_alerts().await;

        let mut cache_write = cache.top_alerts.write().await;
        *cache_write = top_alerts;
    }
}
