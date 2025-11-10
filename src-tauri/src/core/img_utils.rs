use opencv::{
    prelude::*,
    core::{self, Mat, Point, Scalar, Size, Vector},
    imgcodecs, imgproc::{self, LINE_8}, Result, photo
};
// use super::io_control::pause_before_exit;
// use std::fs;
use uuid::Uuid;
use crate::commands::file::CURRENT_PATH;

/* 加载图片 */
pub fn load_image(src_data: &Vec<u8>) -> Result<Mat> {
    // let image = imgcodecs::imread(path, imgcodecs::IMREAD_COLOR)?;
    // if image.empty() {
    //     eprintln!("错误：无法加载图片，请检查路径是否正确。");
    //     pause_before_exit();
    //     Err(opencv::Error::new(-1, "无法加载图片"))
    // } else {
    //     Ok(image)
    // }
    // let buf = fs::read(path).map_err(|e| opencv::Error::new(-1, &format!("无法读取图片文件: {}", e)))?;
    let data = Vector::from_slice(src_data);
    let image = imgcodecs::imdecode(&data, imgcodecs::IMREAD_COLOR)?;
    if image.empty() {
        Err(opencv::Error::new(-1, "无法加载图片"))
    } else {
        Ok(image)
    }
}

/* 导出图片 */
pub fn export_temp_image(real_file_name: &str, image: &Mat) -> Result<String> {
    let mut temp_path = std::env::temp_dir();
    temp_path.push("cutliner");
    if !temp_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&temp_path) {
            return Err(opencv::Error::new(-1, &format!("无法创建临时文件目录: {}", e)));
        }
    }
    let mut real_path = temp_path.clone();
    let file_uuid = Uuid::new_v4();
    let temp_file_path = format!("temp_{}.png", file_uuid);
    temp_path.push(temp_file_path);
    let real_file_path = format!("{}_{}.png", real_file_name, file_uuid);
    real_path.push(real_file_path);
    imgcodecs::imwrite(&temp_path.to_string_lossy().to_string(), image, &Vector::new())?;
    println!("图片已保存到 {}", temp_path.to_string_lossy().to_string());
    std::fs::rename(&temp_path, &real_path).map_err(|e| opencv::Error::new(-1, &format!("无法重命名临时文件: {}", e)))?;
    let mut current_path = CURRENT_PATH.lock().unwrap(); // 更新待保存的文件路径
    *current_path = Some(real_path.to_string_lossy().to_string());
    Ok(real_path.to_string_lossy().to_string())
}


/* 灰度二值化 */
pub fn to_binary(image: &Mat, threshold: f64) -> Result<Mat> {
    let mut gray_image = Mat::default();
    imgproc::cvt_color(image, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;
    let mut binary_image = Mat::default();
    imgproc::threshold(&gray_image, &mut binary_image, threshold, 255.0, imgproc::THRESH_BINARY_INV)?;
    Ok(binary_image)
}   

/* 查找轮廓 */
pub fn find_contours(binary_image: &Mat, enable_inner: bool) -> Result<Vector<Vector<Point>>> {
    let mut contours = Vector::<Vector<Point>>::new();
    imgproc::find_contours(
        &binary_image.clone(),
        &mut contours,
        if enable_inner { imgproc::RETR_TREE } else { imgproc::RETR_EXTERNAL },
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    )?;
    Ok(contours)
}

/* 描绘轮廓 */
pub fn draw_contours_on_mask(img_size: Size, typ: i32,  bg_color: Scalar, contours: &Vector<Vector<Point>>, stroke_color: Scalar, thickness: i32) -> Result<Mat> {
    let mut contour_mask = Mat::new_size_with_default(
        img_size,
        typ, // 单通道灰度图即可
        bg_color, // 背景色
    )?;
    // 创建一个临时的 Vector<Vector<Point>>，每次只放一个轮廓
    let mut single_contour_vec = Vector::<Vector<Point>>::new();

    for contour in contours.iter() {
        single_contour_vec.clear();
        single_contour_vec.push(contour.clone());
        imgproc::draw_contours(
            &mut contour_mask,
            &single_contour_vec,
            -1,
            stroke_color,
            thickness, 
            LINE_8,
            &core::no_array(),
            0,
            Point::new(0, 0)
        )?;
    }
    Ok(contour_mask)
}

/* 外扩轮廓：膨胀掩码 */
pub fn dilate_mask(src_binary: &Mat, expand_pixels: i32) -> Result<Mat> {
    let kernel_size = 2 * expand_pixels + 1;
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        Size::new(kernel_size, kernel_size),
        Point::new(expand_pixels, expand_pixels),
    )?;
    let mut dilated_mask = Mat::default();
    imgproc::dilate(
        src_binary,
        &mut dilated_mask,
        &kernel,
        Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        imgproc::morphology_default_border_value()?,
    )?;
    Ok(dilated_mask)
}

/* 孔洞填充 */
pub fn fill_holes(src_binary: &Mat, size: i32) -> Result<Mat> {
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        Size::new(size, size),
        Point::new(-1, -1),
    )?;
    let mut filled_image = Mat::default();
    imgproc::morphology_ex(
        src_binary,
        &mut filled_image,
        imgproc::MORPH_CLOSE,
        &kernel,
        Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        imgproc::morphology_default_border_value()?,
    )?;
    Ok(filled_image)
}

/* 平滑边角 */
pub fn smooth_edges(src_binary: &Mat, mut size: i32) -> Result<Mat> {
    size = if size % 2 == 0 { size + 1 } else { size }; // 确保为奇数
    let mut blured_image = Mat::default();
    // 高斯模糊产生灰边
    imgproc::gaussian_blur(
        src_binary,
        &mut blured_image,
        Size::new(size, size),
        0.0, 
        0.0, 
        core::BORDER_DEFAULT
    )?;
    // 二值化回去
    let mut smoothed_image = Mat::default();
    imgproc::threshold(&blured_image, &mut smoothed_image, 128.0, 255.0, imgproc::THRESH_BINARY)?;
    Ok(smoothed_image)
}

/* 简化轮廓 */
pub fn simplify_contours(contours: &Vector<Vector<Point>>, epsilon_factor: f64) -> Result<Vector<Vector<Point>>> {
    let mut simplified_contours = Vector::<Vector<Point>>::new();
    for contour in contours.iter() {
        let mut simplified = Vector::<Point>::new();
        let epsilon = epsilon_factor * imgproc::arc_length(&contour, true)?;
        imgproc::approx_poly_dp(&contour, &mut simplified, epsilon, true)?;
        simplified_contours.push(simplified);
    }
    Ok(simplified_contours)
}

// 编码
pub fn mat_to_encoded_vec(mat: &Mat) -> opencv::Result<Vec<u8>> {
    let mut buf = Vector::<u8>::new();
    imgcodecs::imencode(".png", mat, &mut buf, &Vector::new())?;
    Ok(buf.to_vec())
}

// Otsu参考阈值
pub fn get_otsu_threshold(image: &Mat) -> Result<f64> {
    let mut gray_image = Mat::default();
    imgproc::cvt_color(image, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;
    let threshold = imgproc::threshold(&gray_image, &mut Mat::default(), 0.0, 255.0, imgproc::THRESH_BINARY | imgproc::THRESH_OTSU)?;
    Ok(threshold)
}

pub fn create_contours_filled_mask(binary_image: &Mat) -> Result<Mat> {
    let contours = find_contours(binary_image, false)?;
    let mut mask = Mat::new_size_with_default(binary_image.size()?, core::CV_8UC1, Scalar::all(0.0))?;
    // 创建一个临时的 Vector<Vector<Point>>，每次只放一个轮廓
    let mut single_contour_vec = Vector::<Vector<Point>>::new();
    for contour in contours.iter() {
        single_contour_vec.clear();
        single_contour_vec.push(contour.clone());
        imgproc::draw_contours(
            &mut mask,
            &single_contour_vec,
            -1,
            Scalar::all(255.0),
            imgproc::FILLED,
            LINE_8,
            &core::no_array(),
            0,
            Point::new(0, 0)
        )?;
    };
    Ok(mask)
}

// 去背
pub fn remove_background(mat: &Mat, img_binary: &Mat, is_delete_inner: bool) -> Result<Mat> {
    let mut transparent_bg = Mat::new_size_with_default(mat.size()?, core::CV_8UC4, Scalar::all(0.0))?;
    let mut bgra = Mat::default();
    imgproc::cvt_color(mat, &mut bgra, imgproc::COLOR_BGR2BGRA, 0)?;
    if is_delete_inner {
        // 删除内轮廓时，直接使用二值图作为掩码; 使用掩码复制 RGB 部分
        bgra.copy_to_masked(&mut transparent_bg, &img_binary)?;
    } else {
        let mask = create_contours_filled_mask(img_binary)?;
        // 使用掩码复制 RGB 部分
        bgra.copy_to_masked(&mut transparent_bg, &mask)?;
    }
    Ok(transparent_bg)
}

/* 出血 */
pub fn bleed_edges(src_image: &Mat, img_binary: &Mat, expand_pixels: i32) -> Result<Mat> {
    // 1. 原始轮廓
    let original_mask = create_contours_filled_mask(img_binary)?;
    let expanded_mask = dilate_mask(&original_mask, expand_pixels)?;
    let mut white_bg = Mat::new_size_with_default(src_image.size()?, core::CV_8UC3, Scalar::all(255.0))?;
    src_image.copy_to_masked(&mut white_bg, &original_mask)?;
    // 3. 出血掩码
    let mut bleed_mask = Mat::default();
    core::subtract(&expanded_mask, &original_mask, &mut bleed_mask, &core::no_array(), -1)?;
    // 4. 修复
    let mut bleeded_image = Mat::default();
    photo::inpaint(
        &white_bg,
        &bleed_mask,
        &mut bleeded_image,
        1.0,
        photo::INPAINT_TELEA,
    )?;
    Ok(bleeded_image)
}

/* 构建svg */
pub fn build_svg_from_contours(contours: &Vector<Vector<Point>>, img_width: i32, img_height: i32) -> String {
    let mut svg_data = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        img_width, img_height, img_width, img_height
    );
    for contour in contours.iter() {
        if contour.len() < 2 {
            continue; // 忽略点数过少的轮廓
        }
        let mut path_data = String::new();
        for (i, point) in contour.iter().enumerate() {
            if i == 0 {
                path_data += &format!("M {} {}", point.x, point.y);
            } else {
                path_data += &format!(" L {} {}", point.x, point.y);
            }
        }
        path_data += " Z"; // 闭合路径
        svg_data += &format!(
            r#"<path d="{}" fill="none" stroke="black" stroke-width="1"/>"#,
            path_data
        );
    }
    svg_data += "</svg>";
    svg_data
}