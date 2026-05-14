#[cfg(not(target_os = "macos"))]
compile_error!("tokidex is macOS-only");

pub mod app;
pub mod codex_store;
pub mod model;
pub mod ui;
