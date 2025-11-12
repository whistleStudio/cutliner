use crate::core::img_utils;
use opencv::core::{Point, Vector};
use std::sync::{LazyLock, Mutex};
use std::{fs, path::PathBuf};
use tauri::path::BaseDirectory;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

pub static SRC_DATA: LazyLock<Mutex<Option<Vec<u8>>>> = LazyLock::new(|| Mutex::new(None)); // 图片源数据 (上传时记录以便多次处理)
pub static SRC_STEM: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None)); // 图片文件名（不含扩展名）
pub static CURRENT_PATH: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None)); // 保存文件相关
pub static IMAGE_SIZE: LazyLock<Mutex<(i32, i32)>> = LazyLock::new(|| Mutex::new((0, 0))); // 图片尺寸 (宽, 高)
pub static CONTOURS: LazyLock<Mutex<Option<Vector<Vector<Point>>>>> = LazyLock::new(|| Mutex::new(None)); // 轮廓数据 (用于保存svg)

#[tauri::command]
pub fn init(app: tauri::AppHandle, img_name_with_ext: &str) {
    // 获取资源根目录（在 dev 模式下是 src-tauri/assets，打包后是应用内部资源目录）
    let resource_path = app
        .path()
        .resolve(
            &format!("assets/{}", img_name_with_ext),
            BaseDirectory::Resource,
        )
        .expect("failed to resolve resource path");
    let image_data = std::fs::read(resource_path.to_string_lossy().to_string()).unwrap_or_default();
    let mut src_data = SRC_DATA.lock().unwrap();
    *src_data = Some(image_data.clone());
    let path_str = resource_path.to_string_lossy().to_string();
    let _ = app.emit("img-changed", &path_str);
    let mut current_path = CURRENT_PATH.lock().unwrap();
    *current_path = Some(path_str.clone());

    let img_name = match img_name_with_ext.rsplit_once(".") {
        Some((stem, _ext)) => stem.to_string(),
        None => img_name_with_ext.to_string(),
    };
    let mut src_stem = SRC_STEM.lock().unwrap();
    *src_stem = Some(img_name.to_string());
    // let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    println!("Initialized with default resource_path: {}", path_str);
}

#[tauri::command]
pub async fn select_image(app: tauri::AppHandle) -> Result<(Option<String>, f64), String> {
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
            let path_str = path_buf.to_string();
            // 写入临时文件目录 C:\Users\<用户名>\AppData\Local\Temp\cutliner
            let ext = std::path::Path::new(&path_str)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png");
            let mut file_stem = "temp";
            if let Some(stem) = std::path::Path::new(&path_str)
                .file_stem()
                .and_then(|s| s.to_str())
            {
                let mut src_stem = SRC_STEM.lock().unwrap();
                *src_stem = Some(stem.to_string());
                file_stem = stem;
            }
            let temp_file_name = format!("{}_{}.{}", file_stem, Uuid::new_v4(), ext);
            let mut temp_path = std::env::temp_dir();
            temp_path.push("cutliner");
            if !temp_path.exists() {
                if let Err(e) = fs::create_dir_all(&temp_path) {
                    return Err(e.to_string());
                }
            }
            temp_path.push(temp_file_name);
            let temp_path_str = temp_path.to_string_lossy().to_string();
            if let Err(e) = fs::write(&temp_path, &image_data) {
                return Err(e.to_string());
            } else {
                let mut current_path = CURRENT_PATH.lock().unwrap();
                *current_path = Some(temp_path_str.clone());
            }
            let img_original = img_utils::load_image(&image_data).map_err(|e| e.to_string())?;
            let otsu_threshold =
                img_utils::get_otsu_threshold(&img_original).map_err(|e| e.to_string())?;
            // 重新选择新图片时，清空之前的轮廓数据
            let mut contours = CONTOURS.lock().unwrap();
            *contours = None;
            Ok((Some(temp_path_str), otsu_threshold))
        }
        None => Ok((None, 0.0)),
    }
}

#[tauri::command]
pub async fn save_image(app: tauri::AppHandle) -> Result<(), String> {
    let src_stem = SRC_STEM.lock().unwrap().clone(); // 为什么不加clone会报错？
    let file_name = src_stem.as_ref().unwrap_or(&"output".to_string()).clone();
    println!("Saving image as: {}", file_name);
    let contours = CONTOURS.lock().unwrap().clone();
    let ext_list = if contours.is_some() {
        vec!["png", "jpg", "jpeg", "svg", "dxf"]
    } else {
        vec!["png", "jpg", "jpeg"]
    };
    let app_cp = app.clone();
    let res = tokio::task::spawn_blocking(move || {
        app_cp.dialog()
            .file()
            .set_title("图像另存为")
            .add_filter("Image Files", &ext_list)
            .set_file_name(format!("{}.png", file_name))
            .blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Some(save_path) = res {
        let mut target_path = save_path.to_string();
        // 判断扩展名
        let target_ext = std::path::Path::new(&save_path.to_string())
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if target_ext == "svg" {
            let svg_data = generate_svg_data()?;
            fs::write(&target_path, svg_data).map_err(|e| e.to_string())?;
        } else if target_ext == "dxf" {
            let (_, svg_path) = img_utils::generate_temp_dir_file("svg");
            let svg_data = generate_svg_data()?;
            fs::write(&svg_path, svg_data).map_err(|e| e.to_string())?;
            // 调用外部工具 svg2dxf 进行转换
            img_utils::convert_svg_to_dxf(app, &svg_path.to_string_lossy(), &target_path)?;
            // img_utils::generate_dxf_from_contours_r12_compatible(&contours, &target_path).map_err(|e| e.to_string())?;
        } else {
            if target_ext == "" {
                target_path = format!("{}.png", target_path); // 无后缀，默认png格式
            }
            let current_path = CURRENT_PATH.lock().unwrap();
            let path_str = current_path.as_ref().ok_or("当前源路径不存在")?;
            let source_path = PathBuf::from(&path_str);
            fs::copy(&source_path, &target_path).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn generate_svg_data() -> Result<String, String> {
    let image_size = *IMAGE_SIZE.lock().unwrap();
    let contours = CONTOURS
        .lock()
        .unwrap()
        .clone()
        .ok_or("当前没有可保存的轮廓数据")?;
    Ok(img_utils::build_svg_from_contours(
        &contours,
        image_size.0,
        image_size.1,
    ))
}
