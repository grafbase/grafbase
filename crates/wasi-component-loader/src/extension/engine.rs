use gateway_config::WasmConfig;
use wasmtime::{CacheConfig, Engine};

pub(crate) fn build_engine(config: WasmConfig) -> wasmtime::Result<Engine> {
    let mut cfg = wasmtime::Config::new();

    let cache_dir = config
        .cache_path
        .inspect(|path| {
            assert!(path.is_absolute(), "Config.with_absolute_paths() was not called.");
        })
        .or_else(|| {
            let path = std::env::current_dir().ok()?;
            Some(path.join(".grafbase").join("wasm-cache"))
        })
        .unwrap_or_else(|| std::env::temp_dir().join("grafbase-wasm-cache"));

    tracing::debug!("Using Wasm cache dir: {}", cache_dir.display());
    cfg.wasm_component_model(true).async_support(true).cache({
        // Wasmtime seems to have a mechanism to re-use the cache.
        // But it relies on a GIT_REV var which doesn't exist during compilation
        // Furthermore the default behavior with debug assertions is to use last modified
        // time of the current executable, which always changes for integration-tests...
        let dir = cache_dir.join(crate::built_info::CARGO_LOCK_HASH);
        if std::fs::create_dir_all(&dir).is_ok() || std::fs::read_dir(&dir).is_ok() {
            let mut cfg = CacheConfig::new();
            cfg.with_directory(dir);
            wasmtime::Cache::new(cfg).ok()
        } else {
            None
        }
    });

    Engine::new(&cfg)
}
