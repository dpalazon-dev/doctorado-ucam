#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if let Err(error) = research_workbench_core::run() {
        research_workbench_core::adapters::windows::startup::show_error(&error);
        std::process::exit(1);
    }
}
