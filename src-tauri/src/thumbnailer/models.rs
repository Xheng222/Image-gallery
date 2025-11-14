use serde::{Deserialize, Serialize};
use tauri::UriSchemeResponder;

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlType {
    Start,
    Stop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlMessage {
    pub control_type: ControlType,
    pub message: Option<[String; 2]>,
}

/// 传出的请求 (id, file_path)
pub type WorkerRequest = (String, String);

/// 返回的响应 (id, bytes)
pub type WorkerResponse = (String, Vec<u8>);

/// 前端响应者 (request_id, responder)
pub type FrontendResponder = (String, UriSchemeResponder);


