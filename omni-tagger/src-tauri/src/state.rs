use crate::config::AppConfig;
use crate::tagger::Tagger;
use std::sync::Mutex;

pub struct AppState {
    pub tagger: Mutex<Option<Tagger>>,
    pub config: Mutex<AppConfig>,
    pub download_lock: tokio::sync::Mutex<()>,
    pub input_tx: tokio::sync::mpsc::UnboundedSender<Vec<String>>,
    pub active_tasks: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
