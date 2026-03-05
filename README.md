# Studio Native

Tauri v2 native shell for [MYHELLO Studio](https://studio.myhello.io). Loads the production web app in a WebView with native macOS and iOS integrations.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Tauri CLI v2](https://v2.tauri.app/): `cargo install tauri-cli --version "^2"`
- Xcode 15+ (for macOS/iOS builds)
- CocoaPods (for iOS): `brew install cocoapods`

## Development

```bash
# macOS dev (hot-reload)
make dev

# iOS simulator
make ios-dev

# Type-check
make check
```

## Production Builds

```bash
# macOS universal binary (arm64 + x86_64)
make build-universal

# iOS device build
make ios-build
```

## Project Structure

```
src-tauri/
  src/
    lib.rs          # App setup, window config, IPC commands
    menu.rs         # macOS native menu bar
    updater.rs      # Auto-update via GitHub Releases
  plugins/
    pencilkit/      # PencilKit overlay for iPad (Apple Pencil)
  capabilities/     # Tauri v2 security permissions
  tauri.conf.json   # Tauri configuration
  gen/apple/        # Generated Xcode project (iOS)
```

## Releasing

Push a version tag to trigger the release workflow:

```bash
# Update version in src-tauri/tauri.conf.json and Cargo.toml
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Action builds a universal macOS `.dmg`, signs it with Apple Developer credentials, notarizes it, and creates a GitHub Release with auto-updater support.
