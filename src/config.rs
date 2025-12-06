// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperatureUnit {
    #[default]
    Fahrenheit,
    Celsius,
}

impl TemperatureUnit {
    /// Returns a display string for the unit.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fahrenheit => "Fahrenheit",
            Self::Celsius => "Celsius",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Fahrenheit => "°F",
            Self::Celsius => "°C",
        }
    }

    pub fn api_param(&self) -> &'static str {
        match self {
            Self::Fahrenheit => "fahrenheit",
            Self::Celsius => "celsius",
        }
    }
}

/// Tab options for the popup interface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupTab {
    #[default]
    AirQuality,
    Alerts,
    Hourly,
    Forecast,
    Settings,
}

/// Measurement system for non-temperature units (wind speed, visibility, etc.)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementSystem {
    #[default]
    Imperial,
    Metric,
}

impl MeasurementSystem {
    /// Returns a display string for the measurement system.
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Imperial => "Imperial",
            Self::Metric => "Metric",
        }
    }

    /// Returns the wind speed unit label.
    pub fn wind_speed_unit(&self) -> &'static str {
        match self {
            Self::Imperial => "mph",
            Self::Metric => "km/h",
        }
    }

    /// Returns the visibility unit label.
    pub fn visibility_unit(&self) -> &'static str {
        match self {
            Self::Imperial => "mi",
            Self::Metric => "km",
        }
    }

    /// Returns the API parameter for wind speed unit.
    pub fn wind_speed_api_param(&self) -> &'static str {
        match self {
            Self::Imperial => "mph",
            Self::Metric => "kmh",
        }
    }

    /// Converts visibility from meters to the appropriate unit.
    pub fn convert_visibility(&self, meters: f32) -> f32 {
        match self {
            Self::Imperial => meters / 1609.34,
            Self::Metric => meters / 1000.0,
        }
    }
}

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq, Serialize, Deserialize)]
#[version = 1]
pub struct Config {
    pub latitude: f64,
    pub longitude: f64,
    pub location_name: String,
    pub temperature_unit: TemperatureUnit,
    pub measurement_system: MeasurementSystem,
    pub refresh_interval_minutes: u64,
    pub use_auto_location: bool,
    /// Stores the manual location when auto-detect is enabled, so it can be restored.
    pub manual_latitude: Option<f64>,
    pub manual_longitude: Option<f64>,
    pub manual_location_name: Option<String>,
    pub last_updated: Option<i64>,
    /// Last selected tab, restored on popup open.
    pub default_tab: PopupTab,
    /// Enable weather alerts (US only via NWS).
    #[serde(default = "default_alerts_enabled")]
    pub alerts_enabled: bool,
    /// Automatically select units based on detected location.
    #[serde(default = "default_auto_units")]
    pub auto_units: bool,
}

fn default_alerts_enabled() -> bool {
    true
}

fn default_auto_units() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            latitude: 40.7128,
            longitude: -74.0060,
            location_name: "New York, NY, United States".to_string(),
            temperature_unit: TemperatureUnit::default(),
            measurement_system: MeasurementSystem::default(),
            refresh_interval_minutes: 15,
            use_auto_location: false,
            manual_latitude: None,
            manual_longitude: None,
            manual_location_name: None,
            last_updated: None,
            default_tab: PopupTab::default(),
            alerts_enabled: true,
            auto_units: true,
        }
    }
}
