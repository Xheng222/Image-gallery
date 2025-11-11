use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum ControlType {
    Start,
    Stop,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ControlMessage {
    pub control_type: ControlType,
    pub message: Option<[String; 2]>,
}


// 传出的请求 (id, file_path)
pub type WorkerRequest = (String, String);

// 返回的响应 (id, bytes)
pub type WorkerResponse = (String, Vec<u8>);


