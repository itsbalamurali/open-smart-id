## Project Overview

SmartID is a digital identity platform similar to Smart-ID and Mobile-ID. It is a monorepo with multiple components.

## Architecture

- **`api/`** — Rust API server using poem (web framework) and sea-orm (ORM). Runs on port 3000.
- **`app/`** — Flutter mobile application (org: `com.smartid`).
- **`sdks/`** — Client SDKs (planned).
- **`site/`** — Website (planned).

## Common Commands

### API (Rust)

```bash
cd api
cargo build          # Build
cargo run            # Run server (localhost:3000)
cargo check          # Type-check without building
cargo test           # Run tests
cargo test <name>    # Run a single test by name
cargo clippy         # Lint
cargo fmt            # Format code
```

### App (Flutter)

Flutter is managed via **fvm** — always prefix flutter commands with `fvm`:

```bash
cd app
fvm flutter run              # Run the app
fvm flutter build apk        # Build Android APK
fvm flutter build ios        # Build iOS
fvm flutter test             # Run all tests
fvm flutter test test/widget_test.dart  # Run a single test file
fvm flutter analyze          # Lint/static analysis
fvm dart format .            # Format code
```

## Key Dependencies

- **API:** poem 3.1, sea-orm 2.0.0-rc, tokio (Rust edition 2024)
- **App:** Flutter SDK ^3.11.3, flutter_lints for analysis
