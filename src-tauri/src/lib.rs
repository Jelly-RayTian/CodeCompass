mod analysis;
mod commands;
mod db;
mod error;
mod models;
mod platform;
mod scanner;
mod tasks;

use tauri::Manager;

use db::scan_runs::mark_interrupted_runs;
use db::Database;
use platform::database_filename;
use tasks::{AnalysisManager, ScanManager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Resolve the app-data directory, creating it if necessary.
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            // Open (or create) the SQLite database and run migrations.
            let db_path = app_data_dir.join(database_filename());
            let database = Database::open(&db_path)?;

            // Any scan that was still running when the app last exited is now
            // permanently interrupted.
            mark_interrupted_runs(&database)?;

            app.manage(database);
            app.manage(ScanManager::new());
            app.manage(AnalysisManager::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analysis::start_analysis,
            commands::analysis::cancel_analysis,
            commands::analysis::get_file_imports,
            commands::analysis::get_analysis_diagnostics,
            commands::analysis::get_analyzed_files,
            commands::application::get_application_info,
            commands::database::get_database_status,
            commands::workspaces::list_workspaces,
            commands::workspaces::pick_folder,
            commands::workspaces::add_folder,
            commands::workspaces::list_indexed_folders_command,
            commands::workspaces::remove_indexed_folder_command,
            commands::workspaces::start_scan,
            commands::workspaces::cancel_scan,
            commands::workspaces::get_scan_status,
            commands::workspaces::list_workspace_files_command,
            commands::workspaces::get_file_details_command,
            commands::workspaces::list_scan_runs_command,
            commands::workspaces::reveal_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
