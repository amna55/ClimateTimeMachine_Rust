use axum::{routing::get, Json, Router, extract::Extension};
use crate::models::heat_alert::HeatAlert;
use crate::scheduler::heat_alert_scheduler::HeatAlertCache;
use std::sync::Arc;

pub async fn get_heat_alerts(Extension(cache): Extension<Arc<HeatAlertCache>>) -> Json<Vec<HeatAlert>> {
    let alerts = cache.top_alerts.read().await;
    Json(alerts.clone())
}

pub fn routes(cache: Arc<HeatAlertCache>) -> Router {
    Router::new()
        .route("/api/heat_alert", get(get_heat_alerts))
        .layer(Extension(cache))
}
