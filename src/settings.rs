use crate::{capture::CaptureBackend, protocol::TxProfile};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Deutsch,
    English,
    Nederlands,
}
impl Language {
    pub fn t<'a>(self, de: &'a str, en: &'a str, nl: &'a str) -> &'a str {
        match self {
            Self::Deutsch => de,
            Self::English => en,
            Self::Nederlands => nl,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub theme: Theme,
    pub close_to_tray: bool,
    pub keep_history: bool,
    pub capture_backend: CaptureBackend,
    pub profiles: BTreeMap<String, TxProfile>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::Deutsch,
            theme: Theme::System,
            close_to_tray: true,
            keep_history: true,
            capture_backend: CaptureBackend::Auto,
            profiles: BTreeMap::new(),
        }
    }
}
pub fn data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .unwrap_or_else(|| std::env::temp_dir().join("user-config"))
        .join("WinLLDP")
}
pub fn save_json<T: Serialize>(name: &str, value: &T) -> Result<(), String> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("{name}.tmp"));
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    let dst = dir.join(name);
    if dst.exists() {
        std::fs::remove_file(&dst).map_err(|e| e.to_string())?;
    }
    std::fs::rename(tmp, dst).map_err(|e| e.to_string())
}
pub fn load_json<T: serde::de::DeserializeOwned>(name: &str) -> Option<T> {
    let p = data_dir().join(name);
    if std::fs::metadata(&p).ok()?.len() > 4 * 1024 * 1024 {
        return None;
    }
    serde_json::from_slice(&std::fs::read(p).ok()?).ok()
}
