// src/main.rs
mod api;
mod models;
mod scheduler;
mod services;
mod utils;

use crate::scheduler::heat_alert_scheduler::{run_scheduler, HeatAlertCache};
use crate::services::earthengine::EarthEngineService;
use crate::services::tile_config_service::TileConfigService;
use axum::serve;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::spawn;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:8080"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any);

    // Shared cache
    let cache = Arc::new(HeatAlertCache::new());

    // Earth Engine service
    let earth_engine_service =
        Arc::new(EarthEngineService::new("http://localhost:8080".to_string()));

    // Tile Config Service (pass the directory, not a file)
    let tile_config_service =
        Arc::new(TileConfigService::new("C:/Users/amina/app/climate-backend"));

    // Preload existing tile_config_{year}.json files
    if let Ok(mut entries) = fs::read_dir("C:/Users/amina/app/climate-backend").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            if file_name.starts_with("tile_config_") && file_name.ends_with(".json") {
                if let Some(year_str) = file_name
                    .strip_prefix("tile_config_")
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    if let Ok(year) = year_str.parse::<i32>() {
                        if let Err(e) = tile_config_service.load_config_for_year(year).await {
                            eprintln!("Failed to load {}: {}", file_name, e);
                        } else {
                            println!("Preloaded tile config for year {}", year);
                        }
                    }
                }
            }
        }
    }

    // Scheduler
    {
        let cache_clone = cache.clone();
        spawn(async move {
            run_scheduler(cache_clone).await;
        });
    }

    // Build app
    let app = api::routes(
        cache.clone(),
        earth_engine_service.clone(),
        tile_config_service.clone(),
    )
    .layer(cors);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    println!(
        "Server running at http://{}",
        listener.local_addr().unwrap()
    );
    println!("Tile configs loaded from: C:/Users/amina/app/climate-backend");

    serve(listener, app.into_make_service()).await.unwrap();
}
