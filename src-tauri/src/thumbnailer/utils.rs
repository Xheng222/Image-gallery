use tauri_plugin_shell::ShellExt;

use crate::APP_HANDLE;

pub fn start_thumbnailer_worker(server_name: String) {
    if let Some(app_handle) = APP_HANDLE.get() {
            app_handle.shell()
            .sidecar("thumbnailer-worker")
            .unwrap()
            .args([server_name])
            .spawn()
            .expect("Failed to spawn sidecar process");
    }
}

