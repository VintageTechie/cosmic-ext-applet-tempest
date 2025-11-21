# Tempest

A weather applet for COSMIC Desktop with automatic location detection.

## Features

- Real-time weather data from Open-Meteo API (no API key required)
- Current temperature displayed in panel
- Detailed popup showing:
  - Location name and manual refresh button in header
  - Last updated timestamp
  - Current conditions (temperature, feels-like, humidity, wind speed)
  - Sunrise and sunset times
  - Collapsible hourly forecast (next 12 hours)
  - Collapsible 7-day forecast with high/low temperatures
- Configurable settings:
  - Temperature unit (Fahrenheit/Celsius)
  - Location (latitude/longitude)
  - Refresh interval
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

Settings are automatically saved and will persist across sessions. The applet defaults to New York City coordinates (40.7128, -74.0060).

The applet supports automatic location detection via IP geolocation.

## Development

A [justfile](./justfile) is included with common recipes:

- `just build-debug` - Compile with debug profile
- `just check` - Run clippy linter
- `just check-json` - LSP-compatible linter output

## License

GPL-3.0-only - See [LICENSE](./LICENSE)
