use std::{collections::HashMap, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::Duration};
use std::sync::mpsc::{self};
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender, TryRecvError};
use tauri::{UriSchemeResponder, http::{ response}};

use crate::{thumbnailer::{model::*, utils::*}};



pub struct ThumbnailerHandler {
    control_message_tx: IpcSender<ControlMessage>,
    request_tx: IpcSender<WorkerRequest>,
    response_thread_tx: mpsc::Sender<(String, UriSchemeResponder)>,
}

pub struct ResponseHandler {
    response_thread_stop: Arc::<AtomicBool>,
    response_rx: IpcReceiver<WorkerResponse>,
    response_map: Arc<Mutex<HashMap<String, UriSchemeResponder>>>,
}


impl ThumbnailerHandler {
    pub fn connect() -> Self {
        let (server, server_name) = IpcOneShotServer::new().unwrap();
        println!("Thumbnailer ControlMessageHandler server name: {}", server_name);

        // start_thumbnailer_worker(server_name.clone());

        let (_, control_message_tx): (_, IpcSender<ControlMessage>) = server.accept().unwrap();

        println!("Connected to Thumbnailer ControlMessageHandler IPC server.");

        let (request_server, request_server_name) = IpcOneShotServer::new().unwrap();
        let (response_server, response_server_name) = IpcOneShotServer::new().unwrap();
        
        control_message_tx.send(ControlMessage {
            control_type: ControlType::Start,
            message: Some([request_server_name, response_server_name]),
        }).unwrap();

        let (_, request_tx): (_, IpcSender<WorkerRequest>) = request_server.accept().unwrap();
        let (_, response_rx): (_, IpcReceiver<WorkerResponse>) = response_server.accept().unwrap();

        let (response_thread_tx, request_thread_rx) = mpsc::channel();

        let response_thread_stop = Arc::new(AtomicBool::new(false));
        let response_thread_stop_clone = response_thread_stop.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let response_map = Arc::new(Mutex::new(HashMap::new()));
            let response_thread_stop_clone_clone = response_thread_stop_clone.clone();

            let response_handler = ResponseHandler {
                response_thread_stop: response_thread_stop_clone,
                response_rx: response_rx,
                response_map: response_map.clone(),
            };

            tauri::async_runtime::spawn_blocking(move || {
                loop {
                    if response_thread_stop_clone_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    match request_thread_rx.recv_timeout(Duration::from_secs(1)) {
                        Ok((request_id, responder)) => {
                            let mut map = response_map.lock().unwrap();
                            map.insert(request_id, responder);
                        }
                        Err(e) => {
                            match e {
                                mpsc::RecvTimeoutError::Timeout => continue,
                                _ => {
                                    println!("Error receiving request: {}", e);
                                    break;
                                }
                                
                            }
                        }
                    }
                }
            });

            response_handler.run_response_loop();
        });

        println!("Connected to Thumbnailer Worker IPC servers.");
        
        Self {
            control_message_tx: control_message_tx,
            request_tx: request_tx,
            response_thread_tx: response_thread_tx,
        }
    }

    pub fn send_request(&self, request: WorkerRequest, responder: UriSchemeResponder) {
        let request_id = request.0.clone();
        match self.request_tx.send(request) {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to send request ID: {}: {}", request_id, e);
                return;
            }
        }
        self.response_thread_tx.send((request_id, responder)).unwrap();
    }

    pub fn send_stop_signal(&self) {
        self.control_message_tx.send(ControlMessage {
            control_type: ControlType::Stop,
            message: None,
        }).unwrap();

    }
}


impl ResponseHandler {
    pub fn run_response_loop(&self) {
        loop {
            if self.response_thread_stop.load(Ordering::Relaxed) {
                break;
            }
            match self.response_rx.try_recv_timeout(Duration::from_secs(1)) {
                Ok(worker_response) => {
                    if let Some(responder) = self.response_map.lock().unwrap().remove(&worker_response.0) {
                        println!("Responded to request ID: {}", worker_response.0);

                        responder.respond(response::Response::builder()
                            .header("Cache-Control", "max-age=3600")
                            .body(worker_response.1)
                            .expect("failed to build response"));
                    }
                }
                Err(e) => {
                    match e {
                        TryRecvError::Empty => continue,
                        _ => {
                            println!("Error receiving worker response: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        self.response_thread_stop.store(true, Ordering::SeqCst);
        println!("ResponseHandler response loop stopped.");
    }
}








