use once_cell::sync::OnceCell;
use percent_encoding::percent_decode_str;
use serde_json::ser;
use tauri::{AppHandle, Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::{db::{db::ImageDB, models::{ImageData, ImageID}}, thumbnailer::thumbnailer::ThumbnailerHandler};

mod thumbnailer;
mod db;

pub static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

/// 获取图片路径列表
#[tauri::command]
fn get_image_paths() -> Vec<db::models::Image> {
    println!("Getting image paths...");

    let image = db::models::Image {
        id: Some(db::models::ImageID::default()),
        width: 1920,
        height: 1080,
    };

    println!("Image Key: {}", serde_json::to_string(&image.id).unwrap());
    let str = "{\"idx\":4294967295,\"version\":1}";
    let image_id: db::models::ImageID = serde_json::from_str(str).unwrap();
    println!("Parsed ImageID from string: {:?}", image_id);

    let mut asset_dir = Vec::new();
    for _i in 1..=50 {
        asset_dir.push(image.clone());
    }

    asset_dir
}

fn asset_protocol_handler<R: Runtime>(
    _ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    println!("Custom protocol request URL: {}", request.uri().path());
    let decoded_path = percent_decode_str(&request.uri().path()[1..]).decode_utf8_lossy();
    println!("Custom protocol request: {}", decoded_path);
    let image_id: ImageID = serde_json::from_str(&decoded_path).unwrap_or_default();

}

/// ### 初始化全局 AppHandle 和管理器
fn init_app(app_handle: &AppHandle) {
    APP_HANDLE
        .set(app_handle.clone())
        .expect("failed to set global app handle");
    
    // let db = ImageDB::new();
    // app_handle.manage(db);

    // let thumbnailer_handler = ThumbnailerHandler::connect();
    // app_handle.manage(thumbnailer_handler);


}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    tauri::Builder::default()
        .setup(|app| {
                init_app(&app.app_handle());
                Ok(())
            })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .register_asynchronous_uri_scheme_protocol(
                "asset",
                asset_protocol_handler
            )
        .invoke_handler(tauri::generate_handler![
            get_image_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");



}
