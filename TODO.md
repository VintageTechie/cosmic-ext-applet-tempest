# TODO List for Cosmic Weather Applet (Tempest)

## v1.0.0 Release - In Progress

### Critical for Release
- [ ] Take screenshot of applet popup and save to `screenshots/tempest-main.png`
- [ ] Commit all release files to repository
- [ ] Create git tag v1.0.0 and push to GitHub
- [ ] Create GitHub release with release notes
- [ ] Generate cargo-sources.json for Flatpak (or submit without and ask reviewers)
- [ ] Submit to Flathub (see FLATHUB_SUBMISSION.md)

### Post-Flathub Approval
- [ ] Apply for System76 COSMIC Apps featured listing
- [ ] Monitor Flathub PR for reviewer feedback
- [ ] Enable 2FA on GitHub account (required within 1 week of approval)

## Completed for v1.0.0
- [x] Enhanced weather details (humidity, feels-like, sunrise/sunset)
  - Added humidity percentage
  - Added feels-like temperature (apparent temperature)
  - Added sunrise/sunset times with timezone support
- [x] Manual refresh button to UI
- [x] Last updated timestamp display
- [x] Loading spinner for better visual feedback
- [x] Wind direction with compass (N, NE, E, etc.)
- [x] Wind gusts information
- [x] UV index display
- [x] Visibility information
- [x] Pressure information
- [x] Cloud cover percentage
- [x] Add version display in UI
- [x] Add tip me link to Ko-fi in about/settings
- [x] Update version to 1.0.0 in Cargo.toml
- [x] Update metainfo.xml for production release
- [x] Create Flatpak manifest (com.vintagetechie.CosmicExtAppletTempest.json)
- [x] Create CHANGELOG.md
- [x] Create FLATHUB_SUBMISSION.md guide
- [x] Create RELEASE_CHECKLIST.md
- [x] Research COSMIC Applet Store submission process
- [x] Create screenshots directory with instructions

## Optional Enhancements (Future Releases)
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
- Distribution via Flathub → COSMIC Store
- vendor.tar created for offline builds (1GB+, consider .gitignore)

## Quick Reference Files
- See [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) for detailed release steps
- See [FLATHUB_SUBMISSION.md](FLATHUB_SUBMISSION.md) for Flathub submission guide
- See [CHANGELOG.md](CHANGELOG.md) for version history
