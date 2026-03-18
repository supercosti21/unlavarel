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

use services::{get_services, start_service, stop_service};
use projects::{get_projects, add_project, remove_project};
use setup::{check_setup, bootstrap_package_manager, install_stack, mark_setup_complete};

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_services,
            start_service,
            stop_service,
            get_projects,
            add_project,
            remove_project,
            check_setup,
            bootstrap_package_manager,
            install_stack,
            mark_setup_complete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MacEnv");
}
