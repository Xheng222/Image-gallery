use std::path::Path;


pub fn is_image_ext(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
        Some(ext) => matches!(ext.as_str(), "png"|"jpg"|"jpeg"|"gif"|"bmp"|"webp"|"tiff"|"tif"),
        None => false,
    }
}
