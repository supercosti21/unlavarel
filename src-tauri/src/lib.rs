pub mod error;
pub mod platform;
pub mod package_manager;
pub mod service_manager;
pub mod registry;
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

use services::{
    get_services, start_service, stop_service, restart_service,
    start_all_services, stop_all_services, get_service_logs,
};
use projects::{get_projects, add_project, remove_project};
use setup::{check_setup, bootstrap_package_manager, install_stack, mark_setup_complete};
use quickapp::{get_templates, create_app};
use php::{get_php_versions, switch_php_version, get_php_extensions, toggle_php_extension};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Services
            get_services,
            start_service,
            stop_service,
            restart_service,
            start_all_services,
            stop_all_services,
            get_service_logs,
            // Projects
            get_projects,
            add_project,
            remove_project,
            // Setup
            check_setup,
            bootstrap_package_manager,
            install_stack,
            mark_setup_complete,
            // Quick app
            get_templates,
            create_app,
            // PHP
            get_php_versions,
            switch_php_version,
            get_php_extensions,
            toggle_php_extension,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MacEnv");
}
