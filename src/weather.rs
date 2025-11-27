// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

/// Current weather conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentWeather {
    pub temperature: f32,
    pub weathercode: i32,
    pub windspeed: f32,
    pub humidity: i32,
    pub feels_like: f32,
    pub wind_direction: i32,
    pub wind_gusts: f32,
    pub uv_index: f32,
    pub visibility: f32,
    pub pressure: f32,
    pub cloud_cover: i32,
}

/// Daily forecast data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    pub date: String,
    pub temp_max: f32,
    pub temp_min: f32,
    pub weathercode: i32,
    pub sunrise: String,
    pub sunset: String,
}

/// Hourly forecast data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub time: String,
    pub temperature: f32,
    pub weathercode: i32,
    pub precipitation_probability: i32,
}

/// Complete weather data
#[derive(Debug, Clone)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub hourly: Vec<HourlyForecast>,
    pub forecast: Vec<DailyForecast>,
}

/// AQI standard based on region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AqiStandard {
    Us,
    European,
}

/// Current air quality data
#[derive(Debug, Clone)]
pub struct AirQualityData {
    pub aqi: i32,
    pub standard: AqiStandard,
    pub pm2_5: f32,
    pub pm10: f32,
    pub ozone: f32,
    pub nitrogen_dioxide: f32,
    pub carbon_monoxide: f32,
}

/// Open-Meteo API response structure
#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentData,
    hourly: HourlyData,
    daily: DailyData,
}

#[derive(Debug, Deserialize)]
struct CurrentData {
    temperature_2m: f32,
    weathercode: i32,
    windspeed_10m: f32,
    relative_humidity_2m: i32,
    apparent_temperature: f32,
    wind_direction_10m: i32,
    wind_gusts_10m: f32,
    uv_index: f32,
    visibility: f32,
    surface_pressure: f32,
    cloud_cover: i32,
}

#[derive(Debug, Deserialize)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f32>,
    weathercode: Vec<i32>,
    precipitation_probability: Vec<i32>,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    time: Vec<String>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
    weathercode: Vec<i32>,
    sunrise: Vec<String>,
    sunset: Vec<String>,
}

/// Fetches weather data from Open-Meteo API
pub async fn fetch_weather(
    latitude: f64,
    longitude: f64,
    temperature_unit: &str,
    windspeed_unit: &str,
) -> Result<WeatherData, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weathercode,windspeed_10m,relative_humidity_2m,apparent_temperature,wind_direction_10m,wind_gusts_10m,uv_index,visibility,surface_pressure,cloud_cover&hourly=temperature_2m,weathercode,precipitation_probability&daily=temperature_2m_max,temperature_2m_min,weathercode,sunrise,sunset&temperature_unit={}&windspeed_unit={}&timezone=auto&forecast_days=7&forecast_hours=24",
        latitude, longitude, temperature_unit, windspeed_unit
    );

    let response = reqwest::get(&url).await?;
    let data: OpenMeteoResponse = response.json().await?;

    // Process hourly forecast (limit to 12 hours)
    let mut hourly = Vec::new();
    for i in 0..data.hourly.time.len().min(12) {
        hourly.push(HourlyForecast {
            time: data.hourly.time[i].clone(),
            temperature: data.hourly.temperature_2m[i],
            weathercode: data.hourly.weathercode[i],
            precipitation_probability: data.hourly.precipitation_probability[i],
        });
    }

    // Process daily forecast
    let mut forecast = Vec::new();
    for i in 0..data.daily.time.len() {
        forecast.push(DailyForecast {
            date: data.daily.time[i].clone(),
            temp_max: data.daily.temperature_2m_max[i],
            temp_min: data.daily.temperature_2m_min[i],
            weathercode: data.daily.weathercode[i],
            sunrise: data.daily.sunrise[i].clone(),
            sunset: data.daily.sunset[i].clone(),
        });
    }

    Ok(WeatherData {
        current: CurrentWeather {
            temperature: data.current.temperature_2m,
            weathercode: data.current.weathercode,
            windspeed: data.current.windspeed_10m,
            humidity: data.current.relative_humidity_2m,
            feels_like: data.current.apparent_temperature,
            wind_direction: data.current.wind_direction_10m,
            wind_gusts: data.current.wind_gusts_10m,
            uv_index: data.current.uv_index,
            visibility: data.current.visibility,
            pressure: data.current.surface_pressure,
            cloud_cover: data.current.cloud_cover,
        },
        hourly,
        forecast,
    })
}

/// Checks if coordinates fall within Europe
fn is_european_location(latitude: f64, longitude: f64) -> bool {
    // Rough bounding box: lat 35-71, lon -25 to 40
    (35.0..=71.0).contains(&latitude) && (-25.0..=40.0).contains(&longitude)
}

/// Fetches air quality data from Open-Meteo Air Quality API
pub async fn fetch_air_quality(
    latitude: f64,
    longitude: f64,
) -> Result<AirQualityData, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}&current=us_aqi,european_aqi,pm2_5,pm10,ozone,nitrogen_dioxide,carbon_monoxide&timezone=auto",
        latitude, longitude
    );

    let response = reqwest::get(&url).await?;
    let data: AirQualityResponse = response.json().await?;

    let use_european = is_european_location(latitude, longitude);
    let (aqi, standard) = if use_european {
        (
            data.current.european_aqi.unwrap_or(0),
            AqiStandard::European,
        )
    } else {
        (data.current.us_aqi.unwrap_or(0), AqiStandard::Us)
    };

    Ok(AirQualityData {
        aqi,
        standard,
        pm2_5: data.current.pm2_5.unwrap_or(0.0),
        pm10: data.current.pm10.unwrap_or(0.0),
        ozone: data.current.ozone.unwrap_or(0.0),
        nitrogen_dioxide: data.current.nitrogen_dioxide.unwrap_or(0.0),
        carbon_monoxide: data.current.carbon_monoxide.unwrap_or(0.0),
    })
}

/// Open-Meteo Air Quality API response
#[derive(Debug, Deserialize)]
struct AirQualityResponse {
    current: AirQualityCurrentData,
}

#[derive(Debug, Deserialize)]
struct AirQualityCurrentData {
    us_aqi: Option<i32>,
    european_aqi: Option<i32>,
    pm2_5: Option<f32>,
    pm10: Option<f32>,
    ozone: Option<f32>,
    nitrogen_dioxide: Option<f32>,
    carbon_monoxide: Option<f32>,
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
pub async fn search_city(
    city_name: &str,
) -> Result<Vec<LocationResult>, Box<dyn std::error::Error>> {
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

            eprintln!(
                "Auto-detected location: {}, {} ({})",
                lat, lon, location_name
            );
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
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
}

/// Formats ISO timestamp to hour (e.g., "2025-01-20T14:00" -> "2:00 PM")
pub fn format_hour(time_str: &str) -> String {
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(time_str) {
        datetime
            .format("%I:%M %p")
            .to_string()
            .trim_start_matches('0')
            .to_string()
    } else {
        // Fallback: try to extract hour from string like "2025-01-20T14:00"
        if let Some(time_part) = time_str.split('T').nth(1) {
            if let Some(hour_str) = time_part.split(':').next() {
                if let Ok(hour) = hour_str.parse::<u32>() {
                    let (display_hour, period) = if hour == 0 {
                        (12, "AM")
                    } else if hour < 12 {
                        (hour, "AM")
                    } else if hour == 12 {
                        (12, "PM")
                    } else {
                        (hour - 12, "PM")
                    };
                    return format!("{}:00 {}", display_hour, period);
                }
            }
        }
        time_str.to_string()
    }
}

/// Formats ISO timestamp to time (e.g., "2025-01-20T06:30:00" -> "6:30 AM")
pub fn format_time(time_str: &str) -> String {
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(time_str) {
        datetime
            .format("%I:%M %p")
            .to_string()
            .trim_start_matches('0')
            .to_string()
    } else {
        // Fallback: try to extract time from string like "2025-01-20T06:30:00"
        if let Some(time_part) = time_str.split('T').nth(1) {
            let time_components: Vec<&str> = time_part.split(':').collect();
            if time_components.len() >= 2 {
                if let (Ok(hour), Ok(minute)) = (
                    time_components[0].parse::<u32>(),
                    time_components[1].parse::<u32>(),
                ) {
                    let (display_hour, period) = if hour == 0 {
                        (12, "AM")
                    } else if hour < 12 {
                        (hour, "AM")
                    } else if hour == 12 {
                        (12, "PM")
                    } else {
                        (hour - 12, "PM")
                    };
                    return format!("{}:{:02} {}", display_hour, minute, period);
                }
            }
        }
        time_str.to_string()
    }
}

/// Formats date string to readable format (e.g., "2025-11-25" -> "Tue Nov 25")
pub fn format_date(date_str: &str) -> String {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        date.format("%a %b %d").to_string()
    } else {
        date_str.to_string()
    }
}

/// Converts wind direction in degrees to compass direction
pub fn wind_direction_to_compass(degrees: i32) -> &'static str {
    match degrees {
        0..=22 | 338..=360 => "N",
        23..=67 => "NE",
        68..=112 => "E",
        113..=157 => "SE",
        158..=202 => "S",
        203..=247 => "SW",
        248..=292 => "W",
        293..=337 => "NW",
        _ => "N",
    }
}

/// Converts WMO weather codes to freedesktop icon names
/// https://specifications.freedesktop.org/icon-naming-spec/latest/
pub fn weathercode_to_icon_name(code: i32, is_night: bool) -> &'static str {
    match code {
        // Clear sky
        0 => {
            if is_night {
                "weather-clear-night"
            } else {
                "weather-clear"
            }
        }
        // Mainly clear
        1 => {
            if is_night {
                "weather-few-clouds-night"
            } else {
                "weather-few-clouds"
            }
        }
        // Partly cloudy
        2 => {
            if is_night {
                "weather-few-clouds-night"
            } else {
                "weather-few-clouds"
            }
        }
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
        80..=82 => "weather-showers",
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

/// Converts US AQI value to description
pub fn us_aqi_to_description(aqi: i32) -> &'static str {
    match aqi {
        0..=50 => "Good",
        51..=100 => "Moderate",
        101..=150 => "Unhealthy for Sensitive Groups",
        151..=200 => "Unhealthy",
        201..=300 => "Very Unhealthy",
        _ => "Hazardous",
    }
}

/// Converts European AQI value to description
pub fn eu_aqi_to_description(aqi: i32) -> &'static str {
    match aqi {
        0..=20 => "Good",
        21..=40 => "Fair",
        41..=60 => "Moderate",
        61..=80 => "Poor",
        81..=100 => "Very Poor",
        _ => "Extremely Poor",
    }
}

/// Returns AQI description based on standard
pub fn aqi_to_description(aqi: i32, standard: AqiStandard) -> &'static str {
    match standard {
        AqiStandard::Us => us_aqi_to_description(aqi),
        AqiStandard::European => eu_aqi_to_description(aqi),
    }
}

/// Returns label for the AQI standard
pub fn aqi_standard_label(standard: AqiStandard) -> &'static str {
    match standard {
        AqiStandard::Us => "US AQI",
        AqiStandard::European => "EU AQI",
    }
}
