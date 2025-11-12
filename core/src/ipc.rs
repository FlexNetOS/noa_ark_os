//! Inter-process communication (IPC) subsystem

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub type ChannelId = u64;

#[derive(Debug, Clone)]
pub struct Message {
    pub from: u64,
    pub to: u64,
    pub data: Vec<u8>,
}

static MESSAGE_QUEUES: OnceLock<Mutex<HashMap<ChannelId, Vec<Message>>>> = OnceLock::new();

fn message_queues() -> &'static Mutex<HashMap<ChannelId, Vec<Message>>> {
    MESSAGE_QUEUES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Initialize IPC subsystem
pub fn init() -> Result<(), &'static str> {
    println!("[IPC] Initializing inter-process communication...");
    Ok(())
}

fn create_channel_inner(channel_id: ChannelId) -> Result<(), &'static str> {
    let mut queues = message_queues().lock().unwrap();
    queues.insert(channel_id, Vec::new());
    Ok(())
}

fn send_message_inner(channel_id: ChannelId, message: Message) -> Result<(), &'static str> {
    let mut queues = message_queues().lock().unwrap();
    if let Some(queue) = queues.get_mut(&channel_id) {
        queue.push(message);
        Ok(())
    } else {
        Err("Channel not found")
    }
}

fn receive_message_inner(channel_id: ChannelId) -> Option<Message> {
    let mut queues = message_queues().lock().unwrap();
    if let Some(queue) = queues.get_mut(&channel_id) {
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    } else {
        None
    }
}

/// Capability wrapper over IPC primitives.
#[derive(Clone, Default)]
pub struct IpcService;

impl IpcService {
    /// Register a new communication channel.
    pub fn create_channel(&self, channel_id: ChannelId) -> Result<(), &'static str> {
        create_channel_inner(channel_id)
    }

    /// Deliver a message to a channel.
    pub fn send_message(
        &self,
        channel_id: ChannelId,
        message: Message,
    ) -> Result<(), &'static str> {
        send_message_inner(channel_id, message)
    }

    /// Pull the next message from a channel queue.
    pub fn receive_message(&self, channel_id: ChannelId) -> Option<Message> {
        receive_message_inner(channel_id)
    }
}

/// Create a new channel.
pub fn create_channel(channel_id: ChannelId) -> Result<(), &'static str> {
    IpcService::default().create_channel(channel_id)
}

/// Send a message.
pub fn send_message(channel_id: ChannelId, message: Message) -> Result<(), &'static str> {
    IpcService::default().send_message(channel_id, message)
}

/// Receive a message.
pub fn receive_message(channel_id: ChannelId) -> Option<Message> {
    IpcService::default().receive_message(channel_id)
}
