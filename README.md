# Tempest

A weather applet for COSMIC Desktop with automatic location detection.

## Screenshots

| Main | Air Quality | 7-Day Forecast | Settings |
|------|-------------|----------------|----------|
| ![Main](screenshots/tempest-main.png) | ![Air Quality](screenshots/tempest-aiq.png) | ![7-Day](screenshots/tempest-7day.png) | ![Settings](screenshots/tempest-settings.png) |

## Features

- Real-time weather data from Open-Meteo API (no API key required)
- Current temperature and AQI displayed in panel
- Detailed popup with tabbed interface:
  - Current conditions (temperature, feels-like, humidity)
  - Wind information (speed, direction, gusts)
  - UV index, cloud cover, visibility, pressure
  - Sunrise and sunset times
  - **Air Quality tab**: AQI, PM2.5, PM10, Ozone, NO2, CO levels
  - **Hourly tab**: Next 12 hours forecast
  - **7-Day tab**: Weekly forecast with high/low temperatures
  - **Settings tab**: All configuration options
- Automatic location detection via IP geolocation
- Remembers last selected tab between sessions
- Configurable temperature unit (Fahrenheit/Celsius)
- Configurable refresh interval
- Persistent configuration
- Global weather coverage

## Installation

Clone the repository:

```bash
git clone https://github.com/VintageTechie/cosmic-ext-applet-tempest
cd cosmic-ext-applet-tempest
```

Build and install the project:

```bash
just build-release
sudo just install
```

For alternative packaging methods:

- `deb`: run `just build-deb` and `sudo just install-deb`
- `rpm`: run `just build-rpm` and `sudo just install-rpm`

For vendoring, use `just vendor` and `just vendor-build`

## Configuration

Click the applet to open the popup, which includes a settings section where you can:

- Toggle temperature unit (Fahrenheit/Celsius)
- Enter custom latitude and longitude
- Set refresh interval (in minutes)
- View current version
- Support development via Ko-fi

Settings are automatically saved and will persist across sessions. The applet defaults to New York City coordinates (40.7128, -74.0060).

The applet supports automatic location detection via IP geolocation.

## Development

A [justfile](./justfile) is included with common recipes:

- `just build-debug` - Compile with debug profile
- `just check` - Run clippy linter
- `just check-json` - LSP-compatible linter output

## License

GPL-3.0-only - See [LICENSE](./LICENSE)
