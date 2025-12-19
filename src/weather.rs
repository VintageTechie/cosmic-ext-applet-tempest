// SPDX-License-Identifier: GPL-3.0-only

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const USER_AGENT: &str =
    "(cosmic-ext-applet-tempest, https://github.com/VintageTechie/cosmic-ext-applet-tempest)";

/// Shared HTTP client for connection pooling and consistent headers.
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("failed to build HTTP client")
    })
}

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

/// Geographic region for alert provider and AQI standard selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    Us,
    Europe,
    Canada,
    Unknown,
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

/// Weather alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Minor,
    Moderate,
    Severe,
    Extreme,
    Unknown,
}

impl AlertSeverity {
    /// Parses CAP severity string into enum variant.
    /// Handles variations across providers (NWS, MeteoAlarm, ECCC).
    fn from_cap_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minor" => Self::Minor,
            "moderate" => Self::Moderate,
            "severe" | "major" => Self::Severe,
            "extreme" => Self::Extreme,
            _ => Self::Unknown,
        }
    }
}

/// Weather alert from NWS or other sources.
/// Some fields are included for potential future UI enhancements.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Alert {
    pub id: String,
    pub event: String,
    pub severity: AlertSeverity,
    pub urgency: String,
    pub headline: String,
    pub description: String,
    pub instruction: Option<String>,
    pub area_desc: String,
    pub sent: DateTime<Utc>,
    pub expires: DateTime<Utc>,
}

/// NWS API GeoJSON response structure
#[derive(Debug, Deserialize)]
struct NwsAlertsResponse {
    features: Vec<NwsAlertFeature>,
}

#[derive(Debug, Deserialize)]
struct NwsAlertFeature {
    properties: NwsAlertProperties,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NwsAlertProperties {
    id: String,
    event: String,
    severity: Option<String>,
    urgency: Option<String>,
    headline: Option<String>,
    description: Option<String>,
    instruction: Option<String>,
    area_desc: String,
    sent: String,
    expires: Option<String>,
}

/// MeteoAlarm Atom feed response structure
#[derive(Debug, Deserialize)]
struct MeteoAlarmFeed {
    #[serde(rename = "entry", default)]
    entries: Vec<MeteoAlarmEntry>,
}

/// Single alert entry from MeteoAlarm Atom feed
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MeteoAlarmEntry {
    id: String,
    title: Option<String>,
    #[serde(rename = "identifier")]
    cap_identifier: Option<String>,
    #[serde(rename = "event")]
    cap_event: Option<String>,
    #[serde(rename = "severity")]
    cap_severity: Option<String>,
    #[serde(rename = "urgency")]
    cap_urgency: Option<String>,
    #[serde(rename = "areaDesc")]
    cap_area_desc: Option<String>,
    #[serde(rename = "sent")]
    cap_sent: Option<String>,
    #[serde(rename = "expires")]
    cap_expires: Option<String>,
    #[serde(rename = "effective")]
    cap_effective: Option<String>,
    /// Geocode containing EMMA_ID for region filtering
    #[serde(rename = "geocode")]
    cap_geocode: Option<MeteoAlarmGeocode>,
}

/// Geocode element containing EMMA_ID area identifier.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MeteoAlarmGeocode {
    #[serde(rename = "valueName")]
    value_name: Option<String>,
    value: Option<String>,
}

/// ECCC CAP alert response structure (Environment and Climate Change Canada)
#[derive(Debug, Deserialize)]
struct EcccCapAlert {
    identifier: String,
    status: String,
    #[serde(rename = "msgType")]
    msg_type: String,
    sent: String,
    #[serde(rename = "info", default)]
    info_blocks: Vec<EcccCapInfo>,
}

/// Info block from ECCC CAP alert (one per language)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EcccCapInfo {
    language: Option<String>,
    category: Option<String>,
    event: Option<String>,
    urgency: Option<String>,
    severity: Option<String>,
    certainty: Option<String>,
    effective: Option<String>,
    expires: Option<String>,
    headline: Option<String>,
    description: Option<String>,
    instruction: Option<String>,
    #[serde(rename = "area", default)]
    areas: Vec<EcccCapArea>,
}

/// Area element from ECCC CAP alert
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EcccCapArea {
    #[serde(rename = "areaDesc")]
    area_desc: Option<String>,
    polygon: Option<String>,
}

/// Nominatim reverse geocoding response
#[derive(Debug, Deserialize)]
struct NominatimResponse {
    address: Option<NominatimAddress>,
}

/// Address details from Nominatim.
/// Some fields reserved for future use with other European regions.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
    county: Option<String>,
    state: Option<String>,
    #[serde(rename = "ISO3166-2-lvl4")]
    iso_state: Option<String>,
}

/// MeteoAlarm codenames mapping (EMMA_ID -> region name)
#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct MeteoAlarmCodenames {
    codes: std::collections::HashMap<String, String>,
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

    let response = http_client().get(&url).send().await?;
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

/// Checks if coordinates fall within US territory (continental US, Alaska, Hawaii).
/// Excludes Canadian territory by respecting the US-Canada border.
fn is_us_bounds(lat: f64, lon: f64) -> bool {
    // Alaska: lat 51-72, lon -180 to -129
    let alaska = (51.0..=72.0).contains(&lat) && (-180.0..=-129.0).contains(&lon);
    // Hawaii: lat 18-23, lon -161 to -154
    let hawaii = (18.0..=23.0).contains(&lat) && (-161.0..=-154.0).contains(&lon);

    // Continental US with proper northern border respecting Canada:
    // The US-Canada border varies by region:
    // - West (Pacific to Lake of the Woods): 49N
    // - Great Lakes region: follows the lakes (42-47N)
    // - East (St. Lawrence to Atlantic): ~45N
    let continental = if lon < -95.0 {
        // Western US: border at 49N
        (24.0..=49.0).contains(&lat) && (-125.0..=-95.0).contains(&lon)
    } else if lon < -84.0 {
        // Upper Midwest (MN, WI, MI upper): border near 49N for MN,
        // drops to ~46N for Lake Superior region
        (24.0..=46.5).contains(&lat) && (-95.0..=-84.0).contains(&lon)
    } else if lon < -76.0 {
        // Great Lakes / Southern Ontario overlap zone (MI, OH, NY, PA):
        // Lake Erie is at ~42N, Lake Ontario's south shore at ~43.3N
        // Use 43N to exclude Toronto and everything north of the lakes
        (24.0..=43.0).contains(&lat) && (-84.0..=-76.0).contains(&lon)
    } else if lon < -67.0 {
        // Northeast US (NY, VT, NH, MA, CT, RI): St. Lawrence border ~45N
        (24.0..=45.0).contains(&lat) && (-76.0..=-67.0).contains(&lon)
    } else {
        // Maine: border goes up to ~47N
        (24.0..=47.0).contains(&lat) && (-67.0..=-66.0).contains(&lon)
    };

    continental || alaska || hawaii
}

/// Checks if coordinates fall within Canada.
fn is_canada_bounds(lat: f64, lon: f64) -> bool {
    // Canada: lat 41-84, lon -141 to -52
    (41.0..=84.0).contains(&lat) && (-141.0..=-52.0).contains(&lon)
}

/// Checks if coordinates fall within Europe.
fn is_europe_bounds(lat: f64, lon: f64) -> bool {
    // Rough bounding box: lat 35-71, lon -25 to 40
    (35.0..=71.0).contains(&lat) && (-25.0..=40.0).contains(&lon)
}

/// Detects geographic region from coordinates for alert provider selection.
pub fn detect_region(lat: f64, lon: f64) -> Region {
    if is_us_bounds(lat, lon) {
        return Region::Us;
    }
    if is_canada_bounds(lat, lon) {
        return Region::Canada;
    }
    if is_europe_bounds(lat, lon) {
        return Region::Europe;
    }
    Region::Unknown
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

    let response = http_client().get(&url).send().await?;
    let data: AirQualityResponse = response.json().await?;

    let (aqi, standard) = match detect_region(latitude, longitude) {
        Region::Europe => (
            data.current.european_aqi.unwrap_or(0),
            AqiStandard::European,
        ),
        _ => (data.current.us_aqi.unwrap_or(0), AqiStandard::Us),
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
    pub country: String,
}

impl LocationResult {
    fn from_geocoding_result(result: &GeocodingResult) -> Self {
        let country = result.country.clone().unwrap_or_default();
        let display_name = match (&result.admin1, &result.country) {
            (Some(admin), Some(c)) => format!("{}, {}, {}", result.name, admin, c),
            (None, Some(c)) => format!("{}, {}", result.name, c),
            _ => result.name.clone(),
        };

        Self {
            latitude: result.latitude,
            longitude: result.longitude,
            display_name,
            country,
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

    let response = http_client().get(&url).send().await?;
    let data: GeocodingResponse = response.json().await?;

    if let Some(results) = data.results {
        if !results.is_empty() {
            let locations: Vec<LocationResult> = results
                .iter()
                .map(LocationResult::from_geocoding_result)
                .collect();

            tracing::debug!("Found {} location(s) for '{}'", locations.len(), city_name);
            return Ok(locations);
        }
    }

    Err(format!("No results found for '{}'", city_name).into())
}

/// Detects user location automatically using IP-based geolocation.
/// Returns (latitude, longitude, display_name, country).
pub async fn detect_location() -> Result<(f64, f64, String, String), Box<dyn std::error::Error>> {
    let url = "http://ip-api.com/json/?fields=status,lat,lon,city,regionName,country";

    let response = http_client().get(url).send().await?;
    let data: IpApiResponse = response.json().await?;

    if data.status == "success" {
        if let (Some(lat), Some(lon)) = (data.lat, data.lon) {
            let country = data.country.clone().unwrap_or_default();
            let location_name = match (data.city, data.region_name, data.country) {
                (Some(city), _, Some(c)) => format!("{}, {}", city, c),
                (_, Some(region), Some(c)) => format!("{}, {}", region, c),
                (_, _, Some(c)) => c,
                _ => "Unknown".to_string(),
            };

            tracing::debug!(
                "Auto-detected location: {}, {} ({})",
                lat, lon, location_name
            );
            return Ok((lat, lon, location_name, country));
        }
    }

    Err("Failed to detect location from IP address".into())
}

/// Returns true if the country uses imperial units (Fahrenheit, mph, miles).
/// Only US, Liberia, and Myanmar officially use imperial.
pub fn uses_imperial_units(country: &str) -> bool {
    matches!(country, "United States" | "Liberia" | "Myanmar")
}

/// Maps country name to (MeteoAlarm feed slug, ISO country code).
/// Returns None if country is not covered by MeteoAlarm.
fn get_meteoalarm_info(country: &str) -> Option<(&'static str, &'static str)> {
    match country.to_lowercase().as_str() {
        "austria" => Some(("austria", "AT")),
        "belgium" => Some(("belgium", "BE")),
        "bosnia and herzegovina" => Some(("bosnia-herzegovina", "BA")),
        "bulgaria" => Some(("bulgaria", "BG")),
        "croatia" => Some(("croatia", "HR")),
        "cyprus" => Some(("cyprus", "CY")),
        "czechia" | "czech republic" => Some(("czechia", "CZ")),
        "denmark" => Some(("denmark", "DK")),
        "estonia" => Some(("estonia", "EE")),
        "finland" => Some(("finland", "FI")),
        "france" => Some(("france", "FR")),
        "germany" => Some(("germany", "DE")),
        "greece" => Some(("greece", "GR")),
        "hungary" => Some(("hungary", "HU")),
        "iceland" => Some(("iceland", "IS")),
        "ireland" => Some(("ireland", "IE")),
        "israel" => Some(("israel", "IL")),
        "italy" => Some(("italy", "IT")),
        "latvia" => Some(("latvia", "LV")),
        "lithuania" => Some(("lithuania", "LT")),
        "luxembourg" => Some(("luxembourg", "LU")),
        "malta" => Some(("malta", "MT")),
        "moldova" => Some(("moldova", "MD")),
        "montenegro" => Some(("montenegro", "ME")),
        "netherlands" => Some(("netherlands", "NL")),
        "north macedonia" | "macedonia" => Some(("north-macedonia", "MK")),
        "norway" => Some(("norway", "NO")),
        "poland" => Some(("poland", "PL")),
        "portugal" => Some(("portugal", "PT")),
        "romania" => Some(("romania", "RO")),
        "serbia" => Some(("serbia", "RS")),
        "slovakia" => Some(("slovakia", "SK")),
        "slovenia" => Some(("slovenia", "SI")),
        "spain" => Some(("spain", "ES")),
        "sweden" => Some(("sweden", "SE")),
        "switzerland" => Some(("switzerland", "CH")),
        "united kingdom" | "uk" => Some(("united-kingdom", "UK")),
        _ => None,
    }
}

/// Detects country from coordinates using reverse geocoding.
async fn detect_country_from_coords(
    latitude: f64,
    longitude: f64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Use Open-Meteo geocoding API for reverse lookup
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name=&latitude={}&longitude={}&count=1",
        latitude, longitude
    );

    let response = http_client().get(&url).send().await;
    if let Ok(resp) = response {
        if let Ok(data) = resp.json::<GeocodingResponse>().await {
            if let Some(results) = data.results {
                if let Some(first) = results.first() {
                    if let Some(country) = &first.country {
                        return Ok(country.clone());
                    }
                }
            }
        }
    }

    // Fallback: use approximate country from European bounding boxes
    let country = approximate_european_country(latitude, longitude);
    Ok(country.to_string())
}

/// Approximates country from coordinates using bounding boxes.
/// Used as fallback when reverse geocoding fails.
fn approximate_european_country(lat: f64, lon: f64) -> &'static str {
    // Major European countries by rough bounding boxes
    if (47.3..=55.1).contains(&lat) && (5.9..=15.0).contains(&lon) {
        "Germany"
    } else if (41.3..=51.1).contains(&lat) && (-5.1..=9.6).contains(&lon) {
        "France"
    } else if (36.0..=43.8).contains(&lat) && (-9.5..=3.3).contains(&lon) {
        "Spain"
    } else if (36.6..=47.1).contains(&lat) && (6.6..=18.5).contains(&lon) {
        "Italy"
    } else if (49.9..=61.0).contains(&lat) && (-8.6..=1.8).contains(&lon) {
        "United Kingdom"
    } else if (50.8..=53.5).contains(&lat) && (3.4..=7.2).contains(&lon) {
        "Netherlands"
    } else if (49.5..=51.5).contains(&lat) && (2.5..=6.4).contains(&lon) {
        "Belgium"
    } else if (46.4..=49.0).contains(&lat) && (5.9..=10.5).contains(&lon) {
        "Switzerland"
    } else if (46.4..=49.0).contains(&lat) && (9.5..=17.2).contains(&lon) {
        "Austria"
    } else if (49.0..=54.9).contains(&lat) && (14.1..=24.2).contains(&lon) {
        "Poland"
    } else if (55.0..=69.1).contains(&lat) && (4.5..=31.1).contains(&lon) {
        if lon < 10.0 {
            "Norway"
        } else if lon < 24.2 {
            "Sweden"
        } else {
            "Finland"
        }
    } else {
        "Unknown"
    }
}

/// Fetches active weather alerts from the NWS API for US locations.
async fn fetch_nws_alerts(
    latitude: f64,
    longitude: f64,
) -> Result<Vec<Alert>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://api.weather.gov/alerts/active?point={},{}",
        latitude, longitude
    );

    let response = http_client()
        .get(&url)
        .header("Accept", "application/geo+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("NWS API returned status: {}", response.status()).into());
    }

    let data: NwsAlertsResponse = response.json().await?;

    let alerts: Vec<Alert> = data
        .features
        .into_iter()
        .filter_map(|feature| {
            let props = feature.properties;

            let sent = DateTime::parse_from_rfc3339(&props.sent)
                .ok()?
                .with_timezone(&Utc);

            let expires = props
                .expires
                .as_ref()
                .and_then(|e| DateTime::parse_from_rfc3339(e).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| sent + chrono::Duration::hours(24));

            if expires < Utc::now() {
                return None;
            }

            Some(Alert {
                id: props.id,
                event: props.event,
                severity: props
                    .severity
                    .as_deref()
                    .map(AlertSeverity::from_cap_string)
                    .unwrap_or(AlertSeverity::Unknown),
                urgency: props.urgency.unwrap_or_else(|| "Unknown".to_string()),
                headline: props.headline.unwrap_or_default(),
                description: props.description.unwrap_or_default(),
                instruction: props.instruction,
                area_desc: props.area_desc,
                sent,
                expires,
            })
        })
        .collect();

    tracing::debug!("Fetched {} alert(s) from NWS", alerts.len());
    Ok(alerts)
}

/// Resolves the user's EMMA_ID by looking up their location and matching against codenames.
async fn resolve_user_emma_id(
    latitude: f64,
    longitude: f64,
    country_code: &str,
) -> Option<String> {
    // Get location details from Nominatim
    let nominatim_url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json",
        latitude, longitude
    );

    let response = http_client()
        .get(&nominatim_url)
        .send()
        .await
        .ok()?;

    let nominatim: NominatimResponse = response.json().await.ok()?;
    let address = nominatim.address?;

    // Build list of location names to search for (most specific to least)
    let mut search_terms: Vec<String> = Vec::new();

    if let Some(city) = &address.city {
        search_terms.push(city.clone());
        search_terms.push(format!("Stadt {}", city));
    }
    if let Some(town) = &address.town {
        search_terms.push(town.clone());
    }
    if let Some(county) = &address.county {
        search_terms.push(county.clone());
        search_terms.push(format!("Kreis {}", county));
    }
    if let Some(state) = &address.state {
        search_terms.push(state.clone());
    }

    // Fetch MeteoAlarm codenames
    let codenames_url =
        "https://raw.githubusercontent.com/ktrue/Meteoalarm-warning/master/meteoalarm-codenames.json";
    let codenames_response = http_client().get(codenames_url).send().await.ok()?;
    let codenames: MeteoAlarmCodenames = codenames_response.json().await.ok()?;

    // Find matching EMMA_ID for this country
    let country_prefix = country_code.to_uppercase();
    for search_term in &search_terms {
        let search_lower = search_term.to_lowercase();
        for (emma_id, name) in &codenames.codes {
            // Only match codes for the user's country
            if !emma_id.starts_with(&country_prefix) {
                continue;
            }
            if name.to_lowercase().contains(&search_lower)
                || search_lower.contains(&name.to_lowercase())
            {
                tracing::debug!(
                    "Resolved EMMA_ID: {} ({}) for search term '{}'",
                    emma_id, name, search_term
                );
                return Some(emma_id.clone());
            }
        }
    }

    tracing::debug!(
        "Could not resolve EMMA_ID for location: {:?}",
        search_terms.first()
    );
    None
}

/// Fetches active weather alerts from MeteoAlarm for European locations.
async fn fetch_meteoalarm_alerts(
    latitude: f64,
    longitude: f64,
    country: &str,
) -> Result<Vec<Alert>, Box<dyn std::error::Error + Send + Sync>> {
    let (slug, country_code) = match get_meteoalarm_info(country) {
        Some(info) => info,
        None => {
            tracing::debug!("Country '{}' not covered by MeteoAlarm", country);
            return Ok(vec![]);
        }
    };

    // Try to resolve user's specific EMMA_ID for region filtering
    let user_emma_id = resolve_user_emma_id(latitude, longitude, country_code).await;

    let url = format!(
        "https://feeds.meteoalarm.org/feeds/meteoalarm-legacy-atom-{}",
        slug
    );

    let response = http_client().get(&url).send().await?;
    if !response.status().is_success() {
        return Err(format!("MeteoAlarm returned status: {}", response.status()).into());
    }

    let xml_text = response.text().await?;
    let feed: MeteoAlarmFeed = quick_xml::de::from_str(&xml_text)?;

    let alerts: Vec<Alert> = feed
        .entries
        .into_iter()
        .filter_map(|entry| parse_meteoalarm_entry(entry, &user_emma_id))
        .collect();

    tracing::debug!("Fetched {} alert(s) from MeteoAlarm ({})", alerts.len(), country);
    Ok(alerts)
}

/// Parses a MeteoAlarm entry into an Alert struct.
/// Returns None if the entry doesn't match user's EMMA_ID or is expired.
fn parse_meteoalarm_entry(entry: MeteoAlarmEntry, user_emma_id: &Option<String>) -> Option<Alert> {
    let now = Utc::now();

    // Filter by EMMA_ID if we resolved one for the user
    if let Some(user_id) = user_emma_id {
        let entry_emma_id = entry
            .cap_geocode
            .as_ref()
            .and_then(|gc| gc.value.as_ref());

        match entry_emma_id {
            Some(entry_id) if entry_id != user_id => {
                // Entry has an EMMA_ID but it doesn't match user's location
                return None;
            }
            _ => {
                // Either matches or entry has no geocode (include it)
            }
        }
    }

    // Parse sent timestamp
    let sent = entry
        .cap_sent
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(now);

    // Parse expires timestamp
    let expires = entry
        .cap_expires
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| sent + chrono::Duration::hours(24));

    // Skip expired alerts
    if expires < now {
        return None;
    }

    let event = entry
        .cap_event
        .unwrap_or_else(|| "Weather Alert".to_string());

    let headline = entry.title.unwrap_or_else(|| event.clone());

    let severity = entry
        .cap_severity
        .as_deref()
        .map(AlertSeverity::from_cap_string)
        .unwrap_or(AlertSeverity::Unknown);

    Some(Alert {
        id: entry.cap_identifier.unwrap_or(entry.id),
        event,
        severity,
        urgency: entry.cap_urgency.unwrap_or_else(|| "Unknown".to_string()),
        headline,
        description: String::new(), // MeteoAlarm feeds don't include descriptions
        instruction: None,
        area_desc: entry.cap_area_desc.unwrap_or_default(),
        sent,
        expires,
    })
}

/// Maps Canadian province/territory to ECCC weather office codes.
/// Returns the primary office and optionally a secondary office for border regions.
fn get_eccc_office_codes(lat: f64, lon: f64) -> Vec<&'static str> {
    // Office codes based on approximate provincial boundaries
    // CWTO - Ontario Storm Prediction Centre (Toronto)
    // CWUL - Quebec Storm Prediction Centre (Montreal)
    // CWHX - Atlantic Storm Prediction Centre (Dartmouth) - NS, NB, PE, NL
    // CWWG - Prairie Storm Prediction Centre (Winnipeg) - MB, SK
    // CWNT - Prairie and Arctic Storm Prediction Centre (Edmonton) - AB, NT, NU
    // CWVR - Pacific and Yukon Storm Prediction Centre (Vancouver) - BC, YT

    let mut offices = Vec::new();

    // British Columbia: roughly west of -120
    if lon < -114.0 && lat < 60.0 {
        offices.push("CWVR");
    }
    // Yukon: northwest corner
    if lon < -124.0 && lat > 60.0 {
        offices.push("CWVR");
    }
    // Alberta: -120 to -110, south of 60
    if (-120.0..=-110.0).contains(&lon) && lat < 60.0 {
        offices.push("CWNT");
    }
    // Northwest Territories and Nunavut: north of 60
    if lat > 60.0 && lon > -124.0 {
        offices.push("CWNT");
    }
    // Saskatchewan and Manitoba: -110 to -89
    if (-110.0..=-89.0).contains(&lon) && lat < 60.0 {
        offices.push("CWWG");
    }
    // Ontario: -95 to -74
    if (-95.0..=-74.0).contains(&lon) && lat < 56.0 {
        offices.push("CWTO");
    }
    // Quebec: east of -79
    if lon > -79.0 && lat < 55.0 && lon < -57.0 {
        offices.push("CWUL");
    }
    // Atlantic provinces: east of -67 or specific lat/lon ranges
    if lon > -67.0 || (lon > -64.0 && lat < 48.0) {
        offices.push("CWHX");
    }

    // Fallback: if no office matched, return all major offices
    if offices.is_empty() {
        offices.push("CWTO");
    }

    offices
}

/// Checks if a point is inside a polygon using ray casting algorithm.
fn point_in_polygon(lat: f64, lon: f64, polygon_str: &str) -> bool {
    // Parse polygon string: "lat1,lon1 lat2,lon2 lat3,lon3 ..."
    let vertices: Vec<(f64, f64)> = polygon_str
        .split_whitespace()
        .filter_map(|coord| {
            let parts: Vec<&str> = coord.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(lat), Ok(lon)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    return Some((lat, lon));
                }
            }
            None
        })
        .collect();

    if vertices.len() < 3 {
        return false;
    }

    // Ray casting algorithm
    let mut inside = false;
    let n = vertices.len();
    let mut j = n - 1;

    for i in 0..n {
        let (yi, xi) = vertices[i];
        let (yj, xj) = vertices[j];

        if ((yi > lat) != (yj > lat)) && (lon < (xj - xi) * (lat - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }

    inside
}

/// Fetches active weather alerts from ECCC (Environment and Climate Change Canada).
async fn fetch_eccc_alerts(
    latitude: f64,
    longitude: f64,
) -> Result<Vec<Alert>, Box<dyn std::error::Error + Send + Sync>> {
    let offices = get_eccc_office_codes(latitude, longitude);
    let today = chrono::Utc::now().format("%Y%m%d").to_string();
    let client = http_client();

    let mut all_alerts: Vec<Alert> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for office in offices {
        // Fetch directory listing for today's alerts from this office
        let dir_url = format!(
            "https://dd.weather.gc.ca/today/alerts/cap/{}/{}/",
            today, office
        );

        let dir_response = match client.get(&dir_url).send().await {
            Ok(resp) if resp.status().is_success() => resp,
            _ => continue,
        };

        let dir_html = match dir_response.text().await {
            Ok(text) => text,
            Err(_) => continue,
        };

        // Parse hour directories from HTML listing
        let hour_dirs: Vec<String> = dir_html
            .lines()
            .filter_map(|line| {
                if line.contains("href=\"") && line.contains("/\"") {
                    let start = line.find("href=\"")? + 6;
                    let end = line[start..].find('"')? + start;
                    let href = &line[start..end];
                    // Match two-digit hour directories
                    if href.len() == 3 && href.ends_with('/') {
                        let hour = &href[..2];
                        if hour.chars().all(|c| c.is_ascii_digit()) {
                            return Some(hour.to_string());
                        }
                    }
                }
                None
            })
            .collect();

        for hour in hour_dirs {
            let hour_url = format!("{}{}/", dir_url, hour);

            let hour_response = match client.get(&hour_url).send().await {
                Ok(resp) if resp.status().is_success() => resp,
                _ => continue,
            };

            let hour_html = match hour_response.text().await {
                Ok(text) => text,
                Err(_) => continue,
            };

            // Parse CAP file links
            let cap_files: Vec<String> = hour_html
                .lines()
                .filter_map(|line| {
                    if line.contains(".cap\"") {
                        let start = line.find("href=\"")? + 6;
                        let end = line[start..].find('"')? + start;
                        let href = &line[start..end];
                        if href.ends_with(".cap") {
                            return Some(href.to_string());
                        }
                    }
                    None
                })
                .collect();

            for cap_file in cap_files {
                let cap_url = format!("{}{}", hour_url, cap_file);

                let cap_response = match client.get(&cap_url).send().await {
                    Ok(resp) if resp.status().is_success() => resp,
                    _ => continue,
                };

                let cap_xml = match cap_response.text().await {
                    Ok(text) => text,
                    Err(_) => continue,
                };

                if let Some(alert) = parse_eccc_cap(&cap_xml, latitude, longitude, &mut seen_ids) {
                    all_alerts.push(alert);
                }
            }
        }
    }

    tracing::debug!("Fetched {} alert(s) from ECCC", all_alerts.len());
    Ok(all_alerts)
}

/// Parses an ECCC CAP XML document into an Alert struct.
/// Filters by location using polygon containment and deduplicates by identifier.
fn parse_eccc_cap(
    xml: &str,
    lat: f64,
    lon: f64,
    seen_ids: &mut std::collections::HashSet<String>,
) -> Option<Alert> {
    let cap: EcccCapAlert = quick_xml::de::from_str(xml).ok()?;

    // Skip non-actual alerts (test, exercise, etc.)
    if cap.status != "Actual" {
        return None;
    }

    // Skip cancelled alerts
    if cap.msg_type == "Cancel" {
        return None;
    }

    // Find English info block (prefer en-CA)
    let info = cap
        .info_blocks
        .iter()
        .find(|i| {
            i.language
                .as_ref()
                .map(|l| l.starts_with("en"))
                .unwrap_or(false)
        })
        .or_else(|| cap.info_blocks.first())?;

    // Check if user's location is within any of the alert areas
    let mut location_matches = false;
    let mut area_desc = String::new();

    for area in &info.areas {
        if let Some(ref polygon) = area.polygon {
            if point_in_polygon(lat, lon, polygon) {
                location_matches = true;
                area_desc = area.area_desc.clone().unwrap_or_default();
                break;
            }
        }
    }

    // If no polygon matched, skip this alert
    if !location_matches {
        return None;
    }

    let event = info.event.clone().unwrap_or_else(|| "Weather Alert".to_string());

    // Deduplicate by event type + area (ECCC issues updates with new identifiers)
    let dedup_key = format!("{}|{}", event, area_desc);
    if seen_ids.contains(&dedup_key) {
        return None;
    }
    seen_ids.insert(dedup_key);

    // Parse timestamps
    let now = Utc::now();

    let sent = cap
        .sent
        .parse::<DateTime<chrono::FixedOffset>>()
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(now);

    let expires = info
        .expires
        .as_ref()
        .and_then(|s| s.parse::<DateTime<chrono::FixedOffset>>().ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| sent + chrono::Duration::hours(24));

    // Skip expired alerts
    if expires < now {
        return None;
    }

    let headline = info.headline.clone().unwrap_or_else(|| event.clone());

    Some(Alert {
        id: cap.identifier,
        event,
        severity: info
            .severity
            .as_deref()
            .map(AlertSeverity::from_cap_string)
            .unwrap_or(AlertSeverity::Unknown),
        urgency: info.urgency.clone().unwrap_or_else(|| "Unknown".to_string()),
        headline,
        description: info.description.clone().unwrap_or_default(),
        instruction: info.instruction.clone(),
        area_desc,
        sent,
        expires,
    })
}

/// Fetches active weather alerts based on location.
/// Dispatches to appropriate regional API based on detected region.
pub async fn fetch_alerts(
    latitude: f64,
    longitude: f64,
) -> Result<Vec<Alert>, Box<dyn std::error::Error + Send + Sync>> {
    match detect_region(latitude, longitude) {
        Region::Us => fetch_nws_alerts(latitude, longitude).await,
        Region::Europe => {
            let country = detect_country_from_coords(latitude, longitude)
                .await
                .unwrap_or_default();
            fetch_meteoalarm_alerts(latitude, longitude, &country).await
        }
        Region::Canada => fetch_eccc_alerts(latitude, longitude).await,
        Region::Unknown => Ok(vec![]),
    }
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

/// Determines if current time is night (before sunrise or after sunset).
/// Falls back to 6pm-6am if parsing fails.
pub fn is_night_time(sunrise: &str, sunset: &str) -> bool {
    use chrono::{Local, NaiveDateTime, TimeZone, Timelike};

    let now = Local::now();

    // Parse sunrise/sunset times (format: "2025-01-20T06:30")
    let parse_time = |time_str: &str| -> Option<chrono::DateTime<Local>> {
        // Try parsing with seconds first, then without
        NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M"))
            .ok()
            .and_then(|naive| Local.from_local_datetime(&naive).single())
    };

    match (parse_time(sunrise), parse_time(sunset)) {
        (Some(sunrise_time), Some(sunset_time)) => {
            now < sunrise_time || now > sunset_time
        }
        _ => {
            // Fallback to hardcoded 6am-6pm if parsing fails
            let hour = now.hour();
            !(6..18).contains(&hour)
        }
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
