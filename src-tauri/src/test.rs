

use image::{DynamicImage, ColorType, ImageFormat};
use fast_image_resize::{self as fr, FilterType, ImageView, ResizeAlg, ResizeOptions};
use std::error::Error;
use std::path::Path;
use std::io::Cursor;




pub fn generate_thumbnail_optimized() -> Result<Vec<u8>, Box<dyn Error>> {
    let path = Path::new("F:\\Pictures\\Saved Pictures\\target.png");
    let max_width: u32 = 2000;
    let max_height: u32 = 300;
    let mut resizer = fr::Resizer::new();
    let img: DynamicImage = image::open(path)?;

    // 计算耗时
    let start_time = std::time::Instant::now();
    let duration: std::time::Duration;

    // 1. [image] 打开图片
    let src_width = img.width();
    let src_height = img.height();

    // 2. [计算] 计算保持纵横比的新尺寸 (与之前相同)
    let ratio = max_height as f32 / src_height as f32;
    let new_width = ((src_width as f32 * ratio).round() as u32).max(1);
    let new_height = ((src_height as f32 * ratio).round() as u32).max(1);

    
    // 4. [优化] 根据原始像素类型进行匹配，避免转换
    
    // 准备目标缓冲区
    let mut buffer = Cursor::new(Vec::new());

    // 准备编码格式 (与之前相同)
    let format = ImageFormat::WebP;
    let resize_options = ResizeOptions {
        algorithm: ResizeAlg::Convolution(FilterType::Bilinear),
        ..Default::default()
    };

    // [关键优化] 按类型匹配
    match img {
        DynamicImage::ImageRgb8(ref _rgb_img) => {
            let mut dst_image = fr::images::Image::new(new_width, new_height, fr::PixelType::U8x3);
            resizer.resize(&img, &mut dst_image, &resize_options)?;

            duration = start_time.elapsed();

            image::write_buffer_with_format(
                &mut buffer, dst_image.buffer(), new_width, new_height, ColorType::Rgb8, format
            )?;
        },
        DynamicImage::ImageRgba8(ref _rgba_img) => {

            let mut dst_image = fr::images::Image::new(new_width, new_height, fr::PixelType::U8x4);
            resizer.resize(&img, &mut dst_image, &resize_options)?;

            duration = start_time.elapsed();

            image::write_buffer_with_format(
                &mut buffer, dst_image.buffer(), new_width, new_height, ColorType::Rgba8, format
            )?;
        },
        DynamicImage::ImageLuma8(ref luma_img) => { // 灰度图

            let mut dst_image = fr::images::Image::new(new_width, new_height, fr::PixelType::U8);
            resizer.resize(&img, &mut dst_image, &resize_options)?;

            duration = start_time.elapsed();

            image::write_buffer_with_format(
                &mut buffer, dst_image.buffer(), new_width, new_height, ColorType::L8, format
            )?;
        },
        // [回退] 对于其他不常见的格式 (如 LumaA8, Rgb16)，我们回退到简单转换
        _ => {
            let mut dst_image = fr::images::Image::new(new_width, new_height, fr::PixelType::U8x4);
            resizer.resize(&img, &mut dst_image, &resize_options)?;

            duration = start_time.elapsed();

            image::write_buffer_with_format(
                &mut buffer, dst_image.buffer(), new_width, new_height, ColorType::Rgba8, format
            )?;
        }
    }

    // 计算耗时
    // let duration = start_time.elapsed();
    println!("生成缩略图耗时: {:?}", duration);
    
    // 计算缩略图压缩比
    let original_size = std::fs::metadata(path)?.len();
    let thumbnail_size = buffer.get_ref().len() as u64;
    let compression_ratio = 1.0 - thumbnail_size as f64 / original_size as f64;
    println!("原始图片大小: {} bytes", original_size);
    println!("缩略图大小: {} bytes", thumbnail_size);
    println!("缩略图压缩比: {:.2}%", compression_ratio * 100.0);

    match img {
        DynamicImage::ImageRgb8(_) => println!("使用了 RGB8 优化路径"),
        DynamicImage::ImageRgba8(_) => println!("使用了 RGBA8 优化路径"),
        DynamicImage::ImageLuma8(_) => println!("使用了 Luma8 优化路径"),
        _ => println!("使用了回退路径"),
    }

    Ok(buffer.into_inner())
}

