**Rust Commands**
cargo test
cargo build

flutter_rust_bridge_codegen generate `
    --rust-input crate::core::api `
    --rust-root . `
    --dart-output ..\argus_frontend\lib\bridge_generated.dart

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

cargo ndk -t armeabi-v7a -t arm64-v8a -t x86_64 -o ..\argus_frontend\android\app\src\main\jniLibs build --release


**Flutter Commands**
flutter clean
flutter pub get
flutter build apk --release