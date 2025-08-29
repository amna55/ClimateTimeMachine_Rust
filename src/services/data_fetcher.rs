use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct OpenMeteoCurrentWeather {
    temperature: f64, // in Celsius
}

#[derive(Deserialize)]
struct OpenMeteoResponse {
    current_weather: OpenMeteoCurrentWeather,
}

pub async fn get_temperature(lat: f64, lng: f64) -> Result<f64> {
    if lat < -90.0 || lat > 90.0 || lng < -180.0 || lng > 180.0 {
        anyhow::bail!("Invalid coordinates");
    }

    // Build Open-Meteo API URL
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
        lat, lng
    );

    // Make HTTP GET request
    let resp = reqwest::get(&url).await?.json::<OpenMeteoResponse>().await?;

    // Return the current temperature in Celsius
    Ok(resp.current_weather.temperature)
}
