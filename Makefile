.PHONY: dev build build-universal ios-dev ios-build clean check

# macOS development
dev:
	cd src-tauri && cargo tauri dev

# macOS release build (current arch)
build:
	cd src-tauri && cargo tauri build

# macOS universal binary (arm64 + x86_64)
build-universal:
	cd src-tauri && cargo tauri build --target universal-apple-darwin

# iOS simulator
ios-dev:
	cd src-tauri && cargo tauri ios dev

# iOS device build
ios-build:
	cd src-tauri && cargo tauri ios build

# Type-check Rust
check:
	cd src-tauri && cargo check

# Clean build artifacts
clean:
	cd src-tauri && cargo clean

# Generate icons from a source PNG (requires cargo-tauri)
icons:
	cd src-tauri && cargo tauri icon icons/app-icon.png
