use std::sync::Mutex;
use crate::tagger::Tagger;
use crate::config::AppConfig;

pub struct AppState {
    pub tagger: Mutex<Option<Tagger>>,
    pub config: Mutex<AppConfig>,
}
