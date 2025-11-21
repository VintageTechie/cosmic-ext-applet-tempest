# Release 1.0.0 Checklist

This checklist tracks everything needed for the production release of Tempest v1.0.0.

## Completed Tasks

- [x] Update version to 1.0.0 in Cargo.toml
- [x] Update metainfo.xml with production release information
  - Changed type from "desktop-application" to "addon"
  - Added COSMIC category for "Made for COSMIC" section
  - Added comprehensive feature list
  - Added release notes for v1.0.0
  - Added donation URL (Ko-fi)
  - Added screenshots section
- [x] Create Flatpak manifest (com.vintagetechie.CosmicExtAppletTempest.json)
- [x] Create vendor.tar for dependencies (via `just vendor`)
- [x] Create screenshots directory with instructions
- [x] Create CHANGELOG.md
- [x] Create FLATHUB_SUBMISSION.md guide

## Remaining Tasks Before Release

### 1. Screenshots (REQUIRED)
- [ ] Build and install applet locally: `just build-release && just install-local`
- [ ] Restart COSMIC panel or log out/in
- [ ] Take screenshot of applet popup window
- [ ] Save as `screenshots/tempest-main.png`
- [ ] Commit and push screenshot to GitHub

### 2. Git Tag and GitHub Release
- [ ] Commit all changes to main branch
- [ ] Create git tag: `git tag -a v1.0.0 -m "Release version 1.0.0"`
- [ ] Push tag to GitHub: `git push origin v1.0.0`
- [ ] Create GitHub release at https://github.com/VintageTechie/cosmic-ext-applet-tempest/releases
  - Use CHANGELOG.md content for release notes
  - Attach built binaries (optional)

### 3. Generate cargo-sources.json
You have two options:

**Option A: Using Python script (requires aiohttp)**
```bash
python3 flatpak-cargo-generator.py ./Cargo.lock -o cargo-sources.json
```

**Option B: Ask Flathub reviewers**
Submit the PR without cargo-sources.json and ask reviewers to help generate it, or they may accept the vendor.tar approach.

### 4. Test Build (Optional but Recommended)
If you have flatpak-builder installed:
```bash
flatpak-builder --force-clean build-dir com.vintagetechie.CosmicExtAppletTempest.json
flatpak-builder --run build-dir com.vintagetechie.CosmicExtAppletTempest.json cosmic-ext-applet-tempest
```

### 5. Flathub Submission
Follow the steps in [FLATHUB_SUBMISSION.md](FLATHUB_SUBMISSION.md):
1. Fork https://github.com/flathub/flathub
2. Create branch from `new-pr` (NOT master)
3. Add manifest and cargo-sources.json files
4. Create PR to `new-pr` branch
5. Address reviewer feedback
6. Wait for approval

### 6. System76 COSMIC Apps Page (After Flathub Approval)
- [ ] Apply at https://system76.com/cosmic/apps
- [ ] Provide application details and screenshots
- [ ] Wait for System76 review for featured listing

## Files Created for Release

- `Cargo.toml` - Updated to version 1.0.0
- `res/com.vintagetechie.CosmicExtAppletTempest.metainfo.xml` - Updated for applet type and v1.0.0
- `com.vintagetechie.CosmicExtAppletTempest.json` - Flatpak manifest
- `CHANGELOG.md` - Version history
- `FLATHUB_SUBMISSION.md` - Detailed submission guide
- `RELEASE_CHECKLIST.md` - This file
- `screenshots/README.md` - Instructions for taking screenshots
- `vendor.tar` - Vendored Rust dependencies (1GB+)

## Notes

- The metainfo.xml references `screenshots/tempest-main.png` which needs to be created
- The Flatpak manifest references git tag `v1.0.0` which must exist on GitHub
- cargo-sources.json is very large (tracks all Rust dependencies)
- Consider using .gitignore for vendor.tar (it's 1GB+)

## Quick Start: Minimum Steps to Release

1. Take screenshot and save to `screenshots/tempest-main.png`
2. Commit all changes: `git add . && git commit -m "Prepare v1.0.0 release"`
3. Create and push tag: `git tag -a v1.0.0 -m "Release version 1.0.0" && git push origin main v1.0.0`
4. Follow [FLATHUB_SUBMISSION.md](FLATHUB_SUBMISSION.md) to submit to Flathub
