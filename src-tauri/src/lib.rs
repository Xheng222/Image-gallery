use once_cell::sync::OnceCell;
use tauri::{AppHandle, Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::thumbnailer::thumbnailer::ThumbnailerHandler;

mod thumbnailer;
mod models;

pub static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

/**
 获取图片路径列表
*/
#[tauri::command]
fn get_image_paths() -> Vec<String> {
    println!("Getting image paths...");
    let mut asset_dir = Vec::new();
    for i in 1..=50 {
        let path = format!("image{}", i);
        asset_dir.push(path);
    }

    asset_dir
}

fn asset_protocol_handler<R: Runtime>(
    _ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    println!("Custom protocol request URL: {}", request.uri());
    if let Some(app_handle) = APP_HANDLE.get() {
        app_handle.state::<ThumbnailerHandler>()
            .send_request(
                (request.uri().to_string(), String::new()),
                responder,
            );
    }
}

fn init_app(app_handle: &AppHandle) {
    APP_HANDLE
        .set(app_handle.clone())
        .expect("failed to set global app handle");
    
    let thumbnailer_handler = ThumbnailerHandler::connect();
    app_handle.manage(thumbnailer_handler);
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
        .register_asynchronous_uri_scheme_protocol("asset", asset_protocol_handler)
        .invoke_handler(tauri::generate_handler![
            get_image_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");



}
