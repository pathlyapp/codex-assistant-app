#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Some(code) = company_codex_tauri_gui_lib::try_run_token_helper_from_args() {
        std::process::exit(code);
    }
    company_codex_tauri_gui_lib::run();
}
