# TODO List for Cosmic Weather Applet (Tempest)

## Current Work In Progress
- None

## Completed
- [x] Enhanced weather details (humidity, feels-like, sunrise/sunset) - merged to main
  - Added humidity percentage
  - Added feels-like temperature (apparent temperature)
  - Added sunrise/sunset times with timezone support
- [x] Manual refresh button to UI
- [x] Last updated timestamp display
- [x] Loading spinner for better visual feedback

## Additional Weather Features
- [ ] Add wind direction with compass (N, NE, E, etc.)
- [ ] Add wind gusts information
- [ ] Add UV index display
- [ ] Add visibility information
- [ ] Add pressure information
- [ ] Add cloud cover percentage

## Distribution & Packaging
- [ ] Create Flatpak manifest and packaging
- [ ] Research COSMIC Applet Store submission process
- [ ] Submit applet to COSMIC Applet Store

## Optional Enhancements (Future)
- [ ] Multiple saved locations
- [ ] Weather comparison between locations
- [ ] Historical data (yesterday's weather)
- [ ] Weather trends (rising/falling indicators)
- [ ] Weather alerts/warnings
- [ ] Graph views for temperature/precipitation
- [ ] Animated weather icons
- [ ] 12/24 hour format preference
- [ ] Wind speed unit preference (mph, km/h, m/s, knots)
- [ ] Pressure unit preference (hPa, inHg, mmHg)

## Notes
- Using Open-Meteo API (https://api.open-meteo.com)
- Freedesktop icon naming specification for weather icons
- All times should use `timezone=auto` in API calls for local time
