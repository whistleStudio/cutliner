use super::file::{SRC_DATA, SRC_STEM};
use crate::core::img_utils;
use opencv::{
    prelude::*,
    Result, 
    core::{CV_8UC1, CV_8UC3} 
};
use uuid::Uuid;
use crate::commands::file::{IMAGE_SIZE, CONTOURS};


#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImgSolveCfgs {
    // Add fields as necessary
    threshold: u8,
    bleed: u8,
    is_delete_inner: u8,
    smooth: u8,
    offset: u8,
    simplify: u8,
    fill_holes: u8,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    BgRemove,
    ContourOuter,
    ContourAll,
}

#[tauri::command]
pub async fn solve(mode: Mode, cfgs: ImgSolveCfgs) -> Result<String, String> {
    let res = tokio::task::spawn_blocking(move || -> Result<String, String> {
        println!("Received mode: {:?}, cfgs: {:?}", mode, cfgs);
        let src_data = SRC_DATA.lock().unwrap().as_ref().unwrap().clone();
        let src_stem = SRC_STEM.lock().unwrap();
        let file_uuid = Uuid::new_v4();
        let temp_file_name = format!("temp_{}.png", file_uuid);
        let file_name = src_stem.as_ref().unwrap().clone();
        /* 处理图像 */
        let mut img_original = img_utils::load_image(&src_data).map_err(|e| e.to_string())?;
        let mut img_binary = img_utils::to_binary(&img_original, cfgs.threshold as f64).map_err(|e| e.to_string())?;
        // 设置当前图片尺寸
        {
            let mut img_size = IMAGE_SIZE.lock().unwrap();
            *img_size = (img_original.cols(), img_original.rows());
        }
        match mode {
            Mode::BgRemove => {
                if cfgs.bleed > 0 {
                    img_original = img_utils::bleed_edges(&img_original, &img_binary, cfgs.bleed as i32).map_err(|e| e.to_string())?;
                    img_binary = img_utils::to_binary(&img_original, cfgs.threshold as f64).map_err(|e| e.to_string())?;
                }
                img_utils::remove_background(&img_original, &img_binary, cfgs.is_delete_inner > 0).map_err(|e| e.to_string())?;
                // 执行背景移除逻辑
                let img_final = img_utils::remove_background(&img_original, &img_binary, cfgs.is_delete_inner > 0).map_err(|e| e.to_string())?;
                // assert_eq!(img_final.channels(), 4, "错误：尝试编码一个非4通道的图像作为透明PNG！");
                let temp_file_path = img_utils::export_temp_image(&file_name, &img_final).map_err(|e| e.to_string())?;
                // 去背处理不提取轮廓，预览生成后，清空轮廓数
                {
                    let mut stored_contours = CONTOURS.lock().unwrap();
                    *stored_contours = None;
                }
                Ok(temp_file_path)
            },
            Mode::ContourOuter | Mode::ContourAll => {
                // 外轮廓描绘逻辑 平滑+膨胀（偏移）+获取轮廓+简化
                // 全部轮廓描绘逻辑 平滑+填充孔洞+膨胀+获取轮廓+简化
                // 1. 平滑
                let img_smoothed = if cfgs.smooth > 0 {
                    img_utils::smooth_edges(&img_binary, cfgs.smooth as i32).map_err(|e| e.to_string())?
                } else {
                    img_binary
                };
                // 2. 填充孔洞 (仅全部轮廓模式)
                let img_filled = if matches!(mode, Mode::ContourAll) && cfgs.fill_holes > 0 {
                    img_utils::fill_holes(&img_smoothed, cfgs.fill_holes as i32).map_err(|e| e.to_string())?
                } else {
                    img_smoothed
                };
                // 3. 膨胀
                let img_dilated = if cfgs.offset != 0 {
                    img_utils::dilate_mask(&img_filled, cfgs.offset as i32).map_err(|e| e.to_string())?
                } else {
                    img_filled
                };
                // 4. 获取轮廓
                let contours = img_utils::find_contours(&img_dilated, matches!(mode, Mode::ContourAll)).map_err(|e| e.to_string())?;
                // 5. 简化轮廓
                let contours_simplified = if cfgs.simplify > 0 {
                    img_utils::simplify_contours(&contours, cfgs.simplify as f64 / 1000.0).map_err(|e| e.to_string())?
                } else {
                    contours
                };
                {
                    let mut stored_contours = CONTOURS.lock().unwrap();
                    *stored_contours = Some(contours_simplified.clone());
                }
                let img_final = img_utils::draw_contours_on_mask(
                    img_original.size().map_err(|e| e.to_string())?,
                    CV_8UC3,
                    opencv::core::Scalar::new(255.0, 255.0, 255.0, 0.0),
                    &contours_simplified,
                    opencv::core::Scalar::all(0.0),
                    1
                ).map_err(|e| e.to_string())?;
                // Ok(img_utils::mat_to_encoded_vec(&img_final).map_err(|e| e.to_string())?)
                let temp_file_path = img_utils::export_temp_image(&temp_file_name, &img_final).map_err(|e| e.to_string())?;
                Ok(temp_file_path)
            }
            // Mode::ContourAll => {
            //     // 执行所有轮廓描绘逻辑
            //     Ok(vec![])
            // },

        }
    })
    .await
    .map_err(|_| "Thread panicked".to_string())??;
    Ok(res)
} 