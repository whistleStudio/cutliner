use crate::commands::file::{SRC_DATA, SRC_STEM};
use tauri::path::BaseDirectory;
use tauri::{Emitter, Manager};

pub fn ready(app: &tauri::AppHandle, img_name: &str) {
    // 获取资源根目录（在 dev 模式下是 src-tauri/assets，打包后是应用内部资源目录）
    let resource_path = app
        .path()
        .resolve(&format!("assets/{}", img_name), BaseDirectory::Resource)
        .expect("failed to resolve resource path");
    let image_data = std::fs::read(resource_path.to_string_lossy().to_string()).unwrap_or_default();
    let mut src_data = SRC_DATA.lock().unwrap();
    *src_data = Some(image_data.clone());
    let mut src_stem = SRC_STEM.lock().unwrap();
    *src_stem = Some(img_name.to_string());

    let _ = app.emit("img-changed", resource_path.to_str().unwrap());

    println!(
        "Initialized with default resource_path: {}",
        resource_path.to_string_lossy().to_string()
    );
}
