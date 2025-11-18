use super::file::{SRC_DATA, SRC_STEM};
use crate::commands::file::{CONTOURS, IMAGE_SIZE};
use crate::core::img_utils;
use opencv::{
    core::{CV_8UC3},
    prelude::*,
    Result,
};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImgSolveCfgs {
    // Add fields as necessary
    threshold: f64,
    bleed: i32,
    is_delete_inner: u8,
    smooth: i32,
    offset: i32,
    simplify: i32,
    remove_noise_inner: i32,
    remove_noise_outer: i32,
    is_contain_inner: u8,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    BgRemove,
    ExtractContour
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
        let mut img_binary = img_utils::to_binary(&img_original, cfgs.threshold)
            .map_err(|e| e.to_string())?;
        // 设置当前图片尺寸
        {
            let mut img_size = IMAGE_SIZE.lock().unwrap();
            *img_size = (img_original.cols(), img_original.rows());
            let diagonal = ((img_original.cols() as f64).powi(2) + (img_original.rows() as f64).powi(2)).sqrt();
            println!(
                "Image size set to: {:?}, diagonal length: {}",
                *img_size, diagonal
            );
        }
        match mode {
            Mode::BgRemove => {
                // 去背逻辑：出血+（填充内部+胡椒降噪）+简化+平滑+背景移除
                if cfgs.bleed > 0 {
                    img_original = img_utils::bleed_edges(&mut img_original, &img_binary, cfgs.bleed)
                            .map_err(|e| e.to_string())?;
                    img_binary = img_utils::to_binary(&img_original, cfgs.threshold)
                        .map_err(|e| e.to_string())?;
                }
                img_binary = img_utils::remove_noise(&img_binary, cfgs.remove_noise_inner, cfgs.remove_noise_outer, cfgs.is_delete_inner > 0)
                        .map_err(|e| e.to_string())?;  
                if cfgs.simplify > 0 {
                    img_binary = img_utils::simplify_contours_from_binary(&img_binary, cfgs.simplify as f64 / 1000.0)
                        .map_err(|e| e.to_string())?;
                }
                if cfgs.smooth > 0 {
                    img_binary = img_utils::smooth_edges(&img_binary, cfgs.smooth).map_err(|e| e.to_string())?;
                }
                // 执行背景移除逻辑
                let img_final = img_utils::remove_background(
                    &img_original,
                    &img_binary
                )
                .map_err(|e| e.to_string())?;
                // assert_eq!(img_final.channels(), 4, "错误：尝试编码一个非4通道的图像作为透明PNG！");
                let temp_file_path = img_utils::export_temp_image(&file_name, &img_final)
                    .map_err(|e| e.to_string())?;

                // let temp_file_path = img_utils::export_temp_image(&file_name, &img_binary)
                //     .map_err(|e| e.to_string())?;
                // 去背处理不提取轮廓，预览生成后，清空轮廓数
                {
                    let mut stored_contours = CONTOURS.lock().unwrap();
                    *stored_contours = None;
                }
                Ok(temp_file_path)
            }
            Mode::ExtractContour => {
                // 外轮廓描绘逻辑：（填充内部+胡椒降噪）+平滑+膨胀（偏移）+轮廓提取+ 简化
                // 1. 降噪
                img_binary = img_utils::remove_noise(&img_binary, cfgs.remove_noise_inner, cfgs.remove_noise_outer, cfgs.is_contain_inner > 0)
                        .map_err(|e| e.to_string())?;
                // 2. 平滑
                if cfgs.smooth > 0 {
                    img_binary = img_utils::smooth_edges(&img_binary, cfgs.smooth)
                        .map_err(|e| e.to_string())?
                }
                // 3. 膨胀
                if cfgs.offset != 0 {
                    img_binary = img_utils::dilate_mask(&img_binary, cfgs.offset)
                        .map_err(|e| e.to_string())?
                };
                // 4. 获取轮廓
                let mut contours =
                    img_utils::find_contours(&img_binary, cfgs.is_contain_inner > 0)
                        .map_err(|e| e.to_string())?;
                // 5. 简化
                if cfgs.simplify > 0 {
                    contours = img_utils::simplify_contours(&contours, cfgs.simplify as f64 / 1000.0)
                        .map_err(|e| e.to_string())?;    
                }
                
                {
                    let mut stored_contours = CONTOURS.lock().unwrap();
                    *stored_contours = Some(contours.clone());
                }
                let img_final = img_utils::draw_contours_on_mask(
                    img_original.size().map_err(|e| e.to_string())?,
                    CV_8UC3,
                    opencv::core::Scalar::new(255.0, 255.0, 255.0, 0.0),
                    &contours,
                    opencv::core::Scalar::all(0.0),
                    1,
                )
                .map_err(|e| e.to_string())?;
                // Ok(img_utils::mat_to_encoded_vec(&img_final).map_err(|e| e.to_string())?)
                let temp_file_path = img_utils::export_temp_image(&temp_file_name, &img_final)
                    .map_err(|e| e.to_string())?;
                Ok(temp_file_path)
            } // Mode::ContourAll => {
              //     // 执行所有轮廓描绘逻辑
              //     Ok(vec![])
              // },
        }
    })
    .await
    .map_err(|_| "Thread panicked".to_string())??;
    Ok(res)
}
