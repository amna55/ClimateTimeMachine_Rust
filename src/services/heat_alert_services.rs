use crate::models::heat_alert::HeatAlert;
use crate::utils::cities::CITIES;
use chrono::Utc;
use reqwest::Client;

const OPENWEATHER_API_KEY: &str = "dd759e220492384e47590a6108043aaa";

pub async fn fetch_city_temperature(client: &Client, lat: f64, lon: f64) -> Option<f64> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=metric&appid={}",
        lat, lon, OPENWEATHER_API_KEY
    );

    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;

    json["main"]["temp"].as_f64()
}

pub async fn get_top_heat_alerts() -> Vec<HeatAlert> {
    let client = Client::new();
    let mut alerts = Vec::new();

    for city in CITIES.iter() {
        if let Some(temp) = fetch_city_temperature(&client, city.lat, city.lon).await {
            alerts.push(HeatAlert {
                city: city.name.to_string(),
                temperature: temp,
                severity: "".to_string(),
                time_ago: "".to_string(),
            });
        }
    }

    alerts.sort_by(|a, b| b.temperature.partial_cmp(&a.temperature).unwrap());
    let mut top3 = alerts.into_iter().take(3).collect::<Vec<_>>();

    let now = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    for alert in &mut top3 {
        alert.severity = if alert.temperature > 45.0 {
            "extreme".to_string()
        } else if alert.temperature > 40.0 {
            "warning".to_string()
        } else {
            "normal".to_string()
        };
        alert.time_ago = now.clone();
    }

    top3
}
