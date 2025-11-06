use tauri_plugin_dialog::{DialogExt};
use std::fs;
use std::sync::{LazyLock, Mutex};
use crate::core::img_utils;

pub static SRC_DATA: LazyLock<Mutex<Option<Vec<u8>>>> = LazyLock::new(|| Mutex::new(None));

#[tauri::command]
pub async fn select_image(app: tauri::AppHandle) -> Result<(Option<Vec<u8>>, f64), String> {
    let res = tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Image Files", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    match res {
        Some(path_buf) => {
            // let mut path_str = IMAGE_PATH.lock().unwrap();
            // *path_str = Some(path_buf.to_string());
            // let path_str = path_str.as_ref().ok_or("没有选择文件")?;
            let image_data = fs::read(path_buf.to_string()).map_err(|e| e.to_string())?;
            let mut src_data = SRC_DATA.lock().unwrap();
            *src_data = Some(image_data.clone());
            let img_original = img_utils::load_image(&image_data).map_err(|e| e.to_string())?;
            let otsu_threshold = img_utils::get_otsu_threshold(&img_original).map_err(|e| e.to_string())?;
            Ok((Some(image_data), otsu_threshold))
        },
        None => Ok((None, 0.0)),
    }
}

// fn convert_base64(image_path: &str) -> Result<String, String> {
//     let image_data = fs::read(image_path).map_err(|e| e.to_string())?;
//     let base64_string = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
//     Ok(base64_string)
// }