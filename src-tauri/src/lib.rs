pub mod error;
pub mod platform;
pub mod package_manager;
pub mod service_manager;
pub mod registry;
pub mod discovery;
pub mod services;
pub mod setup;
pub mod vhosts;
pub mod dns;
pub mod ssl;
pub mod mail;
pub mod projects;
pub mod quickapp;
pub mod php;
pub mod tray;
pub mod logs;
pub mod settings;
pub mod sharing;
pub mod snapshots;
pub mod database;

use discovery::{discover_services, get_cached_services};
use services::{
    get_services, start_service, stop_service, restart_service,
    start_all_services, stop_all_services, get_service_logs, uninstall_package,
};
use projects::{get_projects, add_project, remove_project};
use setup::{check_setup, bootstrap_package_manager, install_stack, mark_setup_complete, health_check};
use quickapp::{get_templates, create_app};
use php::{get_php_versions, switch_php_version, get_php_extensions, toggle_php_extension};
use logs::{watch_service_logs, unwatch_service_logs};
use settings::{get_settings, save_settings, open_in_editor, open_in_browser};
use sharing::{share_site, stop_sharing, get_sharing_providers};
use snapshots::{create_snapshot, list_snapshots, restore_snapshot, delete_snapshot};
use database::{db_test_connection, db_get_connection, db_list_databases, db_create_database, db_drop_database, db_list_tables, db_describe_table, db_run_query};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Discovery
            discover_services,
            get_cached_services,
            // Services
            get_services,
            start_service,
            stop_service,
            restart_service,
            start_all_services,
            stop_all_services,
            get_service_logs,
            uninstall_package,
            // Projects
            get_projects,
            add_project,
            remove_project,
            // Setup
            check_setup,
            bootstrap_package_manager,
            install_stack,
            mark_setup_complete,
            health_check,
            // Quick app
            get_templates,
            create_app,
            // PHP
            get_php_versions,
            switch_php_version,
            get_php_extensions,
            toggle_php_extension,
            // Logs (real-time)
            watch_service_logs,
            unwatch_service_logs,
            // Settings
            get_settings,
            save_settings,
            open_in_editor,
            open_in_browser,
            // Sharing
            share_site,
            stop_sharing,
            get_sharing_providers,
            // Snapshots
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            delete_snapshot,
            // Database manager
            db_test_connection,
            db_get_connection,
            db_list_databases,
            db_create_database,
            db_drop_database,
            db_list_tables,
            db_describe_table,
            db_run_query,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MacEnv");
}
