use crate::models::ApplicationInfo;

/// Returns compile-time information about the application.
///
/// This is a pure function with no side-effects — it does not touch
/// the database or filesystem, making it trivially safe to call.
#[tauri::command]
pub fn get_application_info() -> ApplicationInfo {
    ApplicationInfo {
        name: "CodeCompass".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_timestamp: env!("BUILD_TIMESTAMP").to_string(),
    }
}
