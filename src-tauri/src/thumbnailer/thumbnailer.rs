use std::{collections::HashMap, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{Receiver, RecvTimeoutError, Sender, channel}}, time::Duration};
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender, TryRecvError};
use tauri::{UriSchemeResponder, http::{response}};
use crate::thumbnailer::{ models::*, utils::*};

pub struct ThumbnailerHandler {
    control_message_tx: IpcSender<ControlMessage>,
    request_tx: IpcSender<WorkerRequest>,
    response_thread_tx: Sender<FrontendResponder>,
}

pub struct ResponseHandler {
    response_thread_stop: Arc::<AtomicBool>,
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

        let (response_thread_tx, request_thread_rx) = channel();

        let response_thread_stop = Arc::new(AtomicBool::new(false));
        let response_thread_stop_clone = response_thread_stop.clone();


        let response_map = Arc::new(Mutex::new(HashMap::new()));

        let response_handler = Arc::new(ResponseHandler {
            response_thread_stop: response_thread_stop_clone,
            response_map: response_map.clone(),
        });

        let response_handler_clone = Arc::clone(&response_handler);

        tauri::async_runtime::spawn_blocking( move || {
            loop {
                if !response_handler.insert_responser(&request_thread_rx) {
                    break;
                }
            }
            response_handler.stop();
            println!("ResponseHandler request thread stopped.");
        });

        tauri::async_runtime::spawn_blocking(move || {
            loop {
                if !response_handler_clone.get_response(&response_rx) {
                    break;
                }
            }
            response_handler_clone.stop();
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
    pub fn get_response(&self, response_rx: &IpcReceiver<WorkerResponse>) -> bool{
        if self.response_thread_stop.load(Ordering::Relaxed) {
            return false;
        }
        match response_rx.try_recv_timeout(Duration::from_secs(1)) {
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
                    TryRecvError::Empty => {},
                    _ => {
                        println!("Error receiving worker response: {}", e);
                        return false;
                    }
                }
            }
        }

        println!("ResponseHandler response loop stopped.");
        return true;
    }

    pub fn insert_responser(&self, request_thread_rx: &Receiver<FrontendResponder>) -> bool {
        if self.response_thread_stop.load(Ordering::Relaxed) {
            return false;
        }
        match request_thread_rx.recv_timeout(Duration::from_secs(1)) {
            Ok((request_id, responder)) => {
                let mut map = self.response_map.lock().unwrap();
                map.insert(request_id, responder);
            }
            Err(e) => {
                match e {
                    RecvTimeoutError::Timeout => {},
                    _ => {
                        println!("Error receiving request: {}", e);
                        return false;
                    }
                }
            }
        }

        return true;
    }

    pub fn stop(&self) {
        self.response_thread_stop.store(true, Ordering::SeqCst);
    }
}








