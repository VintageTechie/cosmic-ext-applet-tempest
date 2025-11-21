# Screenshots for Tempest Weather Applet

This directory should contain screenshots of the Tempest applet for use in the Flathub store listing.

## Required Screenshot

For Flathub submission, you need at least one screenshot showing:
1. The applet in the COSMIC panel (showing temperature icon)
2. The popup window with current weather conditions and forecasts

## How to Take Screenshots

1. Build and install the applet locally:
   ```bash
   just build-release
   just install-local
   ```

2. Restart the COSMIC panel or log out/in to load the applet

3. Take a screenshot of the popup window:
   - Click the applet to open the popup
   - Take a screenshot (use COSMIC screenshot tool or `gnome-screenshot`)
   - Save as `tempest-main.png`

4. Place the screenshot in this directory

## Screenshot Guidelines

- Use PNG format
- Minimum resolution: 1024x600 pixels
- Show the window only (not the entire desktop or wallpaper)
- Ensure good contrast and readability
- Show realistic weather data

## Current Screenshot

- `tempest-main.png` - Main popup window showing weather information (REQUIRED)
