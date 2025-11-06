use opencv::{
    prelude::*,
    core::{self, Mat, Point, Scalar, Size, Vector},
    imgcodecs, imgproc::{self, LINE_8}, Result,
};
// use super::io_control::pause_before_exit;
// use std::fs;


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
pub fn save_image(path: &str, image: &Mat) -> Result<()> {
    imgcodecs::imwrite(path, image, &Vector::new())?;
    println!("图片已保存到 {}", path);
    Ok(())
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

// 去背
pub fn remove_background(mat: &Mat, contours: &Vector<Vector<Point>>) -> Result<Mat> {
    let mut transparent_bg = Mat::new_size_with_default(mat.size()?, core::CV_8UC4, Scalar::all(0.0))?;
    let mut mask = Mat::new_size_with_default(mat.size()?, core::CV_8UC1, Scalar::all(0.0))?;
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
    }
    // 将原图转成 4 通道（BGRA）
    let mut bgra = Mat::new_size_with_default(mat.size()?, core::CV_8UC4, Scalar::all(0.0))?;
    imgproc::cvt_color(&mat, &mut bgra, imgproc::COLOR_BGR2BGRA, 0)?;

    // 使用掩码复制 RGB 部分
    bgra.copy_to_masked(&mut transparent_bg, &mask)?;

    Ok(transparent_bg)
}