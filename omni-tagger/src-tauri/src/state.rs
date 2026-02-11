use crate::config::AppConfig;
use crate::tagger::Tagger;
use std::sync::Mutex;

pub struct AppState {
    pub tagger: Mutex<Option<Tagger>>,
    pub config: Mutex<AppConfig>,
}
