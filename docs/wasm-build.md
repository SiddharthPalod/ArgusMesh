# WASM Build Instructions for ArgusMesh Web

## Prerequisites

1. **Install wasm-pack**:
   ```powershell
   cargo install wasm-pack
   ```

2. **Add wasm32 target**:
   ```powershell
   rustup target add wasm32-unknown-unknown
   ```

3. **Install nightly Rust** (required for `-Z build-std`):
   ```powershell
   rustup toolchain install nightly
   ```

## Build WASM

Navigate to the backend directory and run:

```powershell
cd backend

# Set nightly toolchain for this directory (or use: rustup default nightly)
rustup override set nightly

# Set required Rust flags for WASM
$env:RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"

# Build WASM - outputs to ../argus_frontend/web/pkg
wasm-pack build --target web --out-dir ../argus_frontend/web/pkg --no-typescript

# Switch back to stable (if you changed default)
rustup override set stable
```

## Run Flutter Web App

After building WASM, run the Flutter app with CORS headers:

```powershell
cd ../argus_frontend
flutter run -d chrome --web-header=Cross-Origin-Opener-Policy=same-origin --web-header=Cross-Origin-Embedder-Policy=require-corp
```

## Build for Production

```powershell
flutter build web --release
```

## Troubleshooting

### Error: "the `-Z build-std` flag requires nightly"
- Make sure you're using nightly toolchain: `rustup override set nightly` in the backend directory

### Error: "MIME type 'text/html' is not executable"
- Make sure WASM files are in `web/pkg/` directory
- Check that `backend.js` and `backend_bg.wasm` exist in `web/pkg/`

### Error: "SharedArrayBuffer is not defined"
- Make sure you're running with CORS headers (see Run Flutter Web App section above)
- Or configure your web server to add these headers:
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp`

## Architecture Notes

- **Web Platform**: Uses btleplug in Rust (may be limited by Web Bluetooth API)
- **Android**: Uses Flutter BLE (flutter_blue_plus)
- **Other Platforms**: Uses btleplug in Rust
