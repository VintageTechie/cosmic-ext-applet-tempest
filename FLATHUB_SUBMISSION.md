# Flathub Submission Guide for Tempest

This guide walks through submitting Tempest to Flathub for distribution through the COSMIC Store.

## Prerequisites

Before submitting, ensure you have:

1. Created a git tag for version 1.0.0:
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

2. Taken screenshots:
   - Take a screenshot of the applet popup window
   - Save as `screenshots/tempest-main.png`
   - Add and commit the screenshot to the repository
   - Push to GitHub

3. Generated cargo-sources.json:
   ```bash
   # Method 1: Using flatpak-cargo-generator (if Python dependencies available)
   curl -sSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -o flatpak-cargo-generator.py
   python3 flatpak-cargo-generator.py ./Cargo.lock -o cargo-sources.json

   # Method 2: Use the vendor.tar approach (already created via `just vendor`)
   # The vendor.tar file can be used in the Flatpak manifest instead of cargo-sources.json
   ```

## Files Ready for Submission

The following files are ready in this repository:

- `com.vintagetechie.CosmicExtAppletTempest.json` - Flatpak manifest
- `res/com.vintagetechie.CosmicExtAppletTempest.metainfo.xml` - AppStream metadata
- `res/com.vintagetechie.CosmicExtAppletTempest.desktop` - Desktop entry
- `res/icons/hicolor/scalable/apps/com.vintagetechie.CosmicExtAppletTempest.svg` - Application icon
- `CHANGELOG.md` - Version history
- `screenshots/` - Screenshots directory

## Submission Process

### Step 1: Fork Flathub Repository

1. Go to https://github.com/flathub/flathub
2. Click "Fork" and ensure you keep all branches
3. Clone your fork:
   ```bash
   cd ~/Code
   git clone https://github.com/YOUR_USERNAME/flathub.git
   cd flathub
   ```

### Step 2: Create Submission Branch

```bash
# Checkout the new-pr branch (NOT master)
git checkout new-pr

# Create a new branch for your submission
git checkout -b add-com.vintagetechie.CosmicExtAppletTempest
```

### Step 3: Add Your Application Files

```bash
# Copy the manifest file
cp ~/Code/cosmic-ext-applet-tempest/com.vintagetechie.CosmicExtAppletTempest.json .

# Copy cargo-sources.json (if generated)
cp ~/Code/cosmic-ext-applet-tempest/cargo-sources.json .

# Add files to git
git add com.vintagetechie.CosmicExtAppletTempest.json
git add cargo-sources.json  # if using this method

# Commit
git commit -m "Add com.vintagetechie.CosmicExtAppletTempest"

# Push to your fork
git push origin add-com.vintagetechie.CosmicExtAppletTempest
```

### Step 4: Create Pull Request

1. Go to your fork on GitHub
2. Click "Pull Request"
3. **Important**: Set base branch to `new-pr` (NOT master)
4. Set compare branch to `add-com.vintagetechie.CosmicExtAppletTempest`
5. Title: "Add com.vintagetechie.CosmicExtAppletTempest"
6. In the description, provide:
   ```
   New COSMIC weather applet submission.

   Tempest is a weather applet for COSMIC Desktop that provides:
   - Real-time weather data with automatic location detection
   - Current conditions and forecasts
   - Hourly and 7-day forecasts
   - Configurable units and location

   Application uses libcosmic and follows COSMIC applet conventions.
   License: GPL-3.0-only
   ```

### Step 5: Address Review Feedback

Flathub reviewers will examine your submission and may request changes:

1. Make requested changes to your files locally
2. Commit and push to the same branch:
   ```bash
   git add -u
   git commit -m "Address review feedback"
   git push origin add-com.vintagetechie.CosmicExtAppletTempest
   ```
3. The PR will update automatically

### Step 6: Testing

When reviewers approve for testing, they may comment "bot, build" to trigger test builds.
Monitor the PR for any build failures and address them.

### Step 7: Final Approval

Once approved:
1. Your app will be merged into the Flathub repository
2. You'll receive an invitation to the Flathub organization
3. **Important**: Enable 2FA on your GitHub account within one week
4. You'll have write access to maintain your app

## Alternative: cargo-sources.json Generation

If you cannot generate cargo-sources.json locally, you can:

1. Submit the PR without it initially
2. Ask Flathub reviewers if they can help generate it
3. Or use the flatpak-builder-tools in a Docker container:
   ```bash
   docker run --rm -v $(pwd):/workspace -w /workspace python:3-slim bash -c "
     pip install aiohttp toml &&
     curl -sSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py | python3 - Cargo.lock -o cargo-sources.json
   "
   ```

## Validation Before Submitting

Validate your manifest and metainfo files:

```bash
# Validate AppStream metadata (if appstream-util is installed)
appstream-util validate-relax res/com.vintagetechie.CosmicExtAppletTempest.metainfo.xml

# Lint the Flatpak manifest (requires flatpak-builder-lint)
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest com.vintagetechie.CosmicExtAppletTempest.json
```

## Maintenance After Approval

For future updates:

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Update metainfo.xml releases section
4. Create git tag for new version
5. In your flathub repository fork:
   ```bash
   cd ~/Code/flathub/com.vintagetechie.CosmicExtAppletTempest
   # Update manifest with new version tag
   git commit -am "Update to version X.Y.Z"
   git push origin master
   ```

## Resources

- Flathub Documentation: https://docs.flathub.org/
- Flathub Submission Guide: https://docs.flathub.org/docs/for-app-authors/submission
- AppStream Guidelines: https://www.freedesktop.org/software/appstream/docs/
- COSMIC Apps: https://system76.com/cosmic/apps

## Notes

- The Flatpak manifest uses `type: git` source and references the v1.0.0 tag
- Make sure the tag exists on GitHub before submitting
- cargo-sources.json is large and tracks all Rust dependencies
- Screenshots are referenced from the GitHub repository (must be pushed first)
