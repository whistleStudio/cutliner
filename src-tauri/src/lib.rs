mod commands;
mod core;

use commands::{file, img_solve};
use core::{temp_files};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    temp_files::clean(); // 启动时清空临时文件目录
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            file::init,
            file::select_image,
            file::save_image,
            img_solve::solve
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
