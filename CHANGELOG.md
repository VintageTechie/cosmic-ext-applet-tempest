# Changelog

All notable changes to Tempest will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.1] - 2025-12-12

### Added
- Toggle to show/hide AQI in panel display (Settings tab)

### Changed
- Internal code refactoring for improved maintainability
- Shared HTTP client for better connection pooling
- Extracted helper functions to reduce code duplication

## [1.4.0] - 2025-12-05

### Added
- Auto-select temperature and measurement units based on detected location
- Weather alerts for US locations via NWS API

### Changed
- Sync README with current features

### Fixed
- Packaging configuration in justfile

## [1.3.0] - 2025-11-27

### Added
- Tabbed popup interface replacing collapsible sections
- Night icon detection using actual sunrise/sunset times

### Fixed
- Resolved clippy warnings

## [1.2.0] - 2025-11-26

### Changed
- Remember manual location settings between sessions
- Sync measurement units with temperature unit selection

## [1.1.0] - 2025-11-25

### Added
- Air quality data from Open-Meteo API
- AQI displayed in panel alongside temperature
- Collapsible air quality section with PM2.5, PM10, ozone, NO2, CO
- Auto-detects European locations and uses EU AQI scale

### Changed
- UI polish and cleanup
- Improved date formatting in 7-day forecast
- Added weather icons to forecast rows
- Replaced Unicode arrows with proper icons in collapsible headers

## [1.0.2] - 2025-11-24

### Added
- Automated .deb package releases via GitHub Actions
- install-dev target in justfile

### Fixed
- Remove %F argument causing Flatpak launch failure

## [1.0.1] - 2025-11-24

### Fixed
- Changed metainfo component type to desktop-application for COSMIC Store visibility
- Added com.system76.CosmicApplet provides declaration

## [1.0.0] - 2025-11-21

### Added
- Initial production release
- Real-time weather data from Open-Meteo API (no API key required)
- Automatic location detection via IP geolocation
- Current temperature displayed in COSMIC panel
- Detailed popup window with comprehensive weather information:
  - Location name with manual refresh button in header
  - Last updated timestamp with loading spinner
  - Current conditions (temperature, feels-like, humidity)
  - Wind information (speed, direction compass, gusts)
  - UV index and cloud cover percentage
  - Visibility and atmospheric pressure
  - Sunrise and sunset times with timezone support
  - Collapsible hourly forecast (next 12 hours)
  - Collapsible 7-day forecast with high/low temperatures
- Configuration settings:
  - Temperature unit toggle (Fahrenheit/Celsius)
  - Custom location support (latitude/longitude)
  - Adjustable refresh interval
  - Version display
  - Ko-fi support link for donations
- Persistent configuration storage
- Global weather coverage

[1.6.1]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.6.1
[1.4.0]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.4.0
[1.3.0]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.3.0
[1.2.0]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.2.0
[1.1.0]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.1.0
[1.0.2]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.0.2
[1.0.1]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.0.1
[1.0.0]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.0.0
