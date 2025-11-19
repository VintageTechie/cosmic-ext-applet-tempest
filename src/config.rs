// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

impl Default for TemperatureUnit {
    fn default() -> Self {
        Self::Fahrenheit
    }
}

impl TemperatureUnit {
    pub fn to_string(&self) -> &'static str {
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

    pub fn convert_from_celsius(&self, celsius: f32) -> f32 {
        match self {
            Self::Celsius => celsius,
            Self::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
        }
    }

    pub fn api_param(&self) -> &'static str {
        match self {
            Self::Fahrenheit => "fahrenheit",
            Self::Celsius => "celsius",
        }
    }
}

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct Config {
    pub latitude: f64,
    pub longitude: f64,
    pub temperature_unit: TemperatureUnit,
    pub refresh_interval_minutes: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            latitude: 40.7128,
            longitude: -74.0060,
            temperature_unit: TemperatureUnit::default(),
            refresh_interval_minutes: 15,
        }
    }
}
