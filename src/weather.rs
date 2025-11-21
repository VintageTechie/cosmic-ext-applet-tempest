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

/// IP-API.com response structure for geolocation
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    country: Option<String>,
}

/// Open-Meteo Geocoding API response structure
#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    admin1: Option<String>,
}

/// Location search result for display
#[derive(Debug, Clone)]
pub struct LocationResult {
    pub latitude: f64,
    pub longitude: f64,
    pub display_name: String,
}

impl LocationResult {
    fn from_geocoding_result(result: &GeocodingResult) -> Self {
        let display_name = match (&result.admin1, &result.country) {
            (Some(admin), Some(country)) => format!("{}, {}, {}", result.name, admin, country),
            (None, Some(country)) => format!("{}, {}", result.name, country),
            _ => result.name.clone(),
        };

        Self {
            latitude: result.latitude,
            longitude: result.longitude,
            display_name,
        }
    }
}

/// Searches for a location by city name using Open-Meteo Geocoding API
pub async fn search_city(city_name: &str) -> Result<Vec<LocationResult>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=10&language=en&format=json",
        urlencoding::encode(city_name)
    );

    let response = reqwest::get(&url).await?;
    let data: GeocodingResponse = response.json().await?;

    if let Some(results) = data.results {
        if !results.is_empty() {
            let locations: Vec<LocationResult> = results
                .iter()
                .map(LocationResult::from_geocoding_result)
                .collect();

            eprintln!("Found {} location(s) for '{}'", locations.len(), city_name);
            return Ok(locations);
        }
    }

    Err(format!("No results found for '{}'", city_name).into())
}

/// Detects user location automatically using IP-based geolocation
pub async fn detect_location() -> Result<(f64, f64, String), Box<dyn std::error::Error>> {
    let url = "http://ip-api.com/json/?fields=status,lat,lon,city,regionName,country";

    let response = reqwest::get(url).await?;
    let data: IpApiResponse = response.json().await?;

    if data.status == "success" {
        if let (Some(lat), Some(lon)) = (data.lat, data.lon) {
            let location_name = match (data.city, data.region_name, data.country) {
                (Some(city), _, Some(country)) => format!("{}, {}", city, country),
                (_, Some(region), Some(country)) => format!("{}, {}", region, country),
                (_, _, Some(country)) => country,
                _ => "Unknown".to_string(),
            };

            eprintln!("Auto-detected location: {}, {} ({})", lat, lon, location_name);
            return Ok((lat, lon, location_name));
        }
    }

    Err("Failed to detect location from IP address".into())
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

/// Converts WMO weather codes to freedesktop icon names
/// https://specifications.freedesktop.org/icon-naming-spec/latest/
pub fn weathercode_to_icon_name(code: i32, is_night: bool) -> &'static str {
    match code {
        // Clear sky
        0 => if is_night { "weather-clear-night" } else { "weather-clear" },
        // Mainly clear
        1 => if is_night { "weather-few-clouds-night" } else { "weather-few-clouds" },
        // Partly cloudy
        2 => if is_night { "weather-few-clouds-night" } else { "weather-few-clouds" },
        // Overcast
        3 => "weather-overcast",
        // Fog and depositing rime fog
        45 | 48 => "weather-fog",
        // Drizzle: Light, moderate, and dense intensity
        51 | 53 | 55 => "weather-showers-scattered",
        // Rain: Slight, moderate and heavy intensity
        61 | 63 | 65 => "weather-showers",
        // Snow fall: Slight, moderate, and heavy intensity
        71 | 73 | 75 => "weather-snow",
        // Snow grains
        77 => "weather-snow",
        // Rain showers: Slight, moderate, and violent
        80 | 81 | 82 => "weather-showers",
        // Snow showers slight and heavy
        85 | 86 => "weather-snow",
        // Thunderstorm
        95 => "weather-storm",
        // Thunderstorm with slight and heavy hail
        96 | 99 => "weather-storm",
        // Unknown
        _ => "weather-severe-alert",
    }
}
