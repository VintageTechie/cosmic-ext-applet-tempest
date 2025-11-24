# Changelog

All notable changes to Tempest will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2025-01-24

### Fixed
- Changed metainfo component type from "addon" to "desktop-application" to appear in COSMIC Store's Applets tab
- Added com.system76.CosmicApplet provides declaration for proper COSMIC applet identification

## [1.0.0] - 2025-01-21

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

[1.0.1]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.0.1
[1.0.0]: https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases/tag/v1.0.0
