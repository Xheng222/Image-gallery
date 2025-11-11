use std::sync::{Arc, atomic::AtomicBool};

use ipc_channel::ipc::IpcReceiver;

use crate::thumbnailer::model::ControlMessage;



pub struct ControlMessageHandler {
    control_message_rx: IpcReceiver<ControlMessage>,
    thumbnailer_worker_thread_stop: Arc::<AtomicBool>,
}













