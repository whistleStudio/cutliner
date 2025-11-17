mod commands;
mod core;

use commands::{file, img_solve};
use core::temp_files;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    temp_files::clean(); // 启动时清空临时文件目录

    tauri::Builder::default()
        // .setup(|_|{
        //     let exe_dir = std::env::current_exe()
        //         .ok()
        //         .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        //     // let bin_dir = exe_dir.
        //     if let Some(exe_dir) = &exe_dir {
        //         let bin_dir = exe_dir.join("bin");
        //         let mut env_path = std::env::var("PATH").unwrap_or_default();
        //         env_path = format!("{};{}", bin_dir.to_string_lossy(), env_path);
        //         println!("Original PATH: {}", env_path);
        //         std::env::set_var("PATH", env_path);
        //     }
        //     println!("Executable directory: {:?}", exe_dir);    
        //     Ok(())
        // })
        .plugin(tauri_plugin_shell::init())
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
