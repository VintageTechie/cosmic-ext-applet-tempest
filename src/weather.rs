// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

/// Current weather conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentWeather {
    pub temperature: f32,
    pub weathercode: i32,
    pub windspeed: f32,
}

/// Daily forecast data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    pub date: String,
    pub temp_max: f32,
    pub temp_min: f32,
    pub weathercode: i32,
}

/// Complete weather data
#[derive(Debug, Clone)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub forecast: Vec<DailyForecast>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Open-Meteo API response structure
#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentData,
    daily: DailyData,
}

#[derive(Debug, Deserialize)]
struct CurrentData {
    temperature_2m: f32,
    weathercode: i32,
    windspeed_10m: f32,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    time: Vec<String>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
    weathercode: Vec<i32>,
}

/// Fetches weather data from Open-Meteo API
pub async fn fetch_weather(latitude: f64, longitude: f64, temperature_unit: &str) -> Result<WeatherData, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weathercode,windspeed_10m&daily=temperature_2m_max,temperature_2m_min,weathercode&temperature_unit={}&windspeed_unit=mph&forecast_days=7",
        latitude, longitude, temperature_unit
    );

    let response = reqwest::get(&url).await?;
    let data: OpenMeteoResponse = response.json().await?;

    let mut forecast = Vec::new();
    for i in 0..data.daily.time.len() {
        forecast.push(DailyForecast {
            date: data.daily.time[i].clone(),
            temp_max: data.daily.temperature_2m_max[i],
            temp_min: data.daily.temperature_2m_min[i],
            weathercode: data.daily.weathercode[i],
        });
    }

    Ok(WeatherData {
        current: CurrentWeather {
            temperature: data.current.temperature_2m,
            weathercode: data.current.weathercode,
            windspeed: data.current.windspeed_10m,
        },
        forecast,
        last_updated: chrono::Utc::now(),
    })
}

/// Converts WMO weather codes to human-readable descriptions
pub fn weathercode_to_description(code: i32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Foggy",
        51 | 53 | 55 => "Drizzle",
        61 | 63 | 65 => "Rain",
        71 | 73 | 75 => "Snow",
        77 => "Snow grains",
        80 | 81 | 82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
}
