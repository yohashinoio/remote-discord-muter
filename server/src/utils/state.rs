use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;
use uuid::Uuid;

use super::{
    kind::{MuteKind, RequestKind},
    other::Watcher,
};

pub struct AppState {
    pub request_senders: Mutex<HashMap<Uuid, mpsc::Sender<RequestKind>>>,
    pub mute_setting_senders: Arc<tokio::sync::Mutex<HashMap<Uuid, mpsc::Sender<MuteKind>>>>,
    pub watchers: Mutex<HashMap<Uuid, Watcher>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            request_senders: Mutex::new(HashMap::new()),
            mute_setting_senders: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            watchers: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_request_sender(&self, uuid: &Uuid) -> Option<mpsc::Sender<RequestKind>> {
        self.request_senders
            .lock()
            .unwrap()
            .get(uuid)
            .map(|x| x.clone())
    }
}
