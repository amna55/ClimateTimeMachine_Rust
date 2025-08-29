// src/services/earthengine.rs

use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::{Result, Context};

#[derive(Debug, Serialize)]
struct LSTRequest {
    bbox: Vec<f64>,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize, Deserialize)] // <- Added Serialize
pub struct LSTResponse {
    pub url: String,
    pub date_range: String,
    pub units: String,
}

pub struct EarthEngineService {
    client: Client,
    python_service_url: String,
}

impl EarthEngineService {
    pub fn new(python_service_url: String) -> Self {
        Self {
            client: Client::new(),
            python_service_url,
        }
    }

    pub async fn get_lst_data(
        &self,
        bbox: Vec<f64>,
        width: u32,
        height: u32,
    ) -> Result<LSTResponse> {
        let request = LSTRequest {
            bbox,
            width,
            height,
        };

        let response = self.client
            .post(&format!("{}/get-lst-tile", self.python_service_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Python service")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Python service returned error: {}",
                error_text
            ));
        }

        let lst_response = response
            .json::<LSTResponse>()
            .await
            .context("Failed to parse Python service response")?;

        Ok(lst_response)
    }
}
