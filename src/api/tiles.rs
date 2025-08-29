use crate::services::tile_config_service::TileConfigService;
use axum::{
    extract::{Json as AxumJson, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;

#[derive(Serialize)]
struct TileUrlsResponse {
    status: String,
    data: Option<TileUrlsData>,
    message: Option<String>,
}

#[derive(Serialize)]
struct TileUrlsData {
    lst_tile_url: String,
    anomaly_tile_url: String,
    absolute_anomaly_tile_url: String,
}

#[derive(Deserialize)]
struct GenerateRequest {
    year: i32,
}

#[derive(Serialize)]
struct GenerateResponse {
    status: String,
    message: String,
    data: Option<TileUrlsData>, // <-- Return URLs directly
}

#[derive(Deserialize)]
struct UrlsQuery {
    year: i32,
}

pub fn routes(tile_config_service: Arc<TileConfigService>) -> Router {
    Router::new()
        .route("/urls", get(get_tile_urls))
        .route("/health", get(tile_health_check))
        .route("/generate", post(generate_tiles))
        .with_state(tile_config_service)
}

// GET tile URLs (optional fallback)
async fn get_tile_urls(
    State(tile_config_service): State<Arc<TileConfigService>>,
    Query(query): Query<UrlsQuery>,
) -> Json<TileUrlsResponse> {
    let year = query.year;

    if let Some((lst_tile_url, anomaly_tile_url, absolute_tile_url)) =
        tile_config_service.get_tile_urls_for_year(year).await
    {
        Json(TileUrlsResponse {
            status: "success".to_string(),
            data: Some(TileUrlsData {
                lst_tile_url: lst_tile_url,
                anomaly_tile_url: anomaly_tile_url,
                absolute_anomaly_tile_url: absolute_tile_url,
            }),
            message: None,
        })
    } else {
        Json(TileUrlsResponse {
            status: "error".to_string(),
            data: None,
            message: Some(format!("Tile config not available yet for year {}", year)),
        })
    }
}

async fn tile_health_check(
    State(tile_config_service): State<Arc<TileConfigService>>,
) -> Json<serde_json::Value> {
    let loaded_years = tile_config_service.get_loaded_years().await;
    Json(serde_json::json!({
        "status": "ok",
        "tile_config_loaded_years": loaded_years,
        "endpoints": {
            "get_urls": "/tiles/urls?year={year}",
            "generate": "/tiles/generate",
            "health": "/tiles/health"
        }
    }))
}

// POST generate tiles AND return URLs
// POST generate tiles AND return URLs
async fn generate_tiles(
    State(tile_config_service): State<Arc<TileConfigService>>,
    AxumJson(payload): AxumJson<GenerateRequest>,
) -> Json<GenerateResponse> {
    use std::env;

    let year = payload.year;
    let script_path = format!(
        "{}/scripts/gee_rust.py",
        env::current_dir().unwrap().display()
    );

    // Run Python with --year
    let output = Command::new("python")
        .arg(script_path)
        .arg("--year")
        .arg(year.to_string())
        .output();

    match output {
        Ok(out) if out.status.success() => {
            // Reload JSON config (with retries)
            let mut retries = 3;
            let mut urls = None;

            while retries > 0 {
                let _ = tile_config_service.load_config_for_year(year).await;

                if let Some((lst_url, anomaly_url, absolute_url)) =
                    tile_config_service.get_tile_urls_for_year(year).await
                {
                    urls = Some((lst_url, anomaly_url, absolute_url));
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                retries -= 1;
            }

            if let Some((lst_tile_url, anomaly_tile_url, absolute_tile_url)) = urls {
                Json(GenerateResponse {
                    status: "success".to_string(),
                    message: format!("Tiles generated for year {}", year),
                    data: Some(TileUrlsData {
                        lst_tile_url: lst_tile_url,
                        anomaly_tile_url: anomaly_tile_url,
                        absolute_anomaly_tile_url: absolute_tile_url,
                    }),
                })
            } else {
                Json(GenerateResponse {
                    status: "error".to_string(),
                    message: format!("Tiles generated but URLs not found for year {}", year),
                    data: None,
                })
            }
        }
        Ok(out) => Json(GenerateResponse {
            status: "error".to_string(),
            message: format!("Python failed: {}", String::from_utf8_lossy(&out.stderr)),
            data: None,
        }),
        Err(e) => Json(GenerateResponse {
            status: "error".to_string(),
            message: format!("Failed to run Python: {}", e),
            data: None,
        }),
    }
}
