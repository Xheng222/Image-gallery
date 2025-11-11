use std::{error::Error, fs, result, thread::sleep, time::Duration};

use tauri::{AppHandle, Runtime, UriSchemeContext, UriSchemeResponder, http::Response, ipc::IpcResponse};

mod test;
mod thumbnailer;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/**
 获取图片路径列表
*/
#[tauri::command]
fn get_image_paths(app: AppHandle) -> Vec<String> {
    println!("Getting image paths...");
    // vec!["asset://image1.png".to_string(), "asset://image2.png".to_string()]
    let mut asset_dir = Vec::new();
    for i in 1..=50 {
        let path = format!("asset://image{}.png", i);
        asset_dir.push(path);
    }
    asset_dir
}

fn asset_protocol_handler<R: Runtime>(
    _ctx: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) { // <-- 已修复
    println!("Custom protocol request URL: {}", request.uri());

    let runtime = tauri::async_runtime::handle();
    runtime.spawn_blocking(move || {
        let result = match test::generate_thumbnail_optimized() {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error generating thumbnail: {}", e);
                Vec::new()
            }
        };
        
        responder.respond(Response::builder()
            // .header("Content-Type", mime)
            .header("Cache-Control", "max-age=3600")
            .body(result)
            .expect("failed to build response"));

    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    // for i in 0..10 {
    //     println!("Test run {}", i);
    //     let _ = test::generate_thumbnail_optimized();
    // }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .register_asynchronous_uri_scheme_protocol("asset", asset_protocol_handler)
        .invoke_handler(tauri::generate_handler![
            get_image_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");



}
