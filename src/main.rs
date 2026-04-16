#![allow(dead_code)]
#![allow(unused_variables)]

fn main() -> app_blocker::utils::error::AppResult<()> {
    if let Err(e) = app_blocker::utils::logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }

    tracing::info!("AppBlocker v{} starting...", app_blocker::VERSION);

    if let Err(e) = validate_environment() {
        tracing::error!("Environment validation failed: {}", e);
        std::process::exit(1);
    }

    let state = std::sync::Arc::new(parking_lot::RwLock::new(app_blocker::core::state::AppState::default()));
    let mut engine = app_blocker::core::engine::Engine::new(state.clone());
    
    if let Err(e) = engine.run() {
        tracing::error!("Engine error: {}", e);
        engine.shutdown();
        std::process::exit(1);
    }

    tracing::info!("AppBlocker shut down gracefully");
    Ok(())
}

fn validate_environment() -> app_blocker::utils::error::AppResult<()> {
    app_blocker::config::env_loader::load_env()?;
    app_blocker::config::validator::validate_permissions()?;
    Ok(())
}
