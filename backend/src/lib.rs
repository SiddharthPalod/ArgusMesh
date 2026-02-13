mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
pub mod core;
pub mod crypto;
pub mod error;
pub mod ffi;
pub mod routing;
pub mod sim;
pub mod storage;
pub mod transport;
pub mod services;
pub mod repository;
pub mod factory;
pub mod builder;
pub use core::api::*;

use std::sync::Once;

static LOGGER_INIT: Once = Once::new();

/// Initialise the Android logger so that `log::info!` etc. show up in logcat
/// under the tag "RustArgus". Safe to call multiple times; only the first call
/// has an effect.
pub fn ensure_logging() {
    LOGGER_INIT.call_once(|| {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("RustArgus"),
        );
        log::info!("RustArgus logger initialised");
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn bam_core_version() -> i32 {
    1
}
