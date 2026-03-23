# SmartID

A digital identity platform similar to Smart-ID and Mobile-ID.

## Structure

| Directory | Description |
|-----------|-------------|
| `api/`    | Rust API server (poem + sea-orm) |
| `app/`    | Flutter mobile application |
| `sdks/`   | Client SDKs |
| `site/`   | Website |

## Getting Started

### API

```bash
cd api
cargo run
```

The API server starts on `http://localhost:3000`.

### App

```bash
cd app
fvm flutter run
```

## Tech Stack

- **API:** Rust, poem 3.1, sea-orm 2.0, tokio
- **App:** Flutter (managed with fvm)
