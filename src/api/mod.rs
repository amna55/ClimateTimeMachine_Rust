use crate::scheduler::heat_alert_scheduler::HeatAlertCache;
use crate::services::earthengine::EarthEngineService;
use crate::services::tile_config_service::TileConfigService;
use axum::Router;
use std::sync::Arc;

pub mod climate;
pub mod heat_alert;
pub mod tiles;

pub fn routes(
    cache: Arc<HeatAlertCache>,
    earth_engine_service: Arc<EarthEngineService>,
    tile_config_service: Arc<TileConfigService>,
) -> Router {
    Router::new()
        .nest("/climate", climate::routes(earth_engine_service))
        .nest("/heat_alert", heat_alert::routes(cache))
        .nest("/tiles", tiles::routes(tile_config_service))
}
