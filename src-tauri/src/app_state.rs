//! État partagé de l'application, exposé aux commandes Tauri via
//! `tauri::State` et initialisé une fois au démarrage (`app.manage(...)`).

use std::sync::Mutex;

use crate::audio::AudioCapture;
use crate::hotkey::{HotkeyRegistry, OsHotkeyBackend};

pub struct AppState {
    pub audio: Mutex<AudioCapture>,
    pub hotkeys: Mutex<HotkeyRegistry>,
    /// `None` tant que le backend natif n'a pas encore été initialisé sur le
    /// thread principal (voir `setup_global_hotkey` dans `lib.rs`) ; doit
    /// rester vivant pendant toute la durée de vie de l'application pour
    /// que les raccourcis restent actifs.
    pub hotkey_backend: Mutex<Option<OsHotkeyBackend>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            audio: Mutex::new(AudioCapture::new()),
            hotkeys: Mutex::new(HotkeyRegistry::new()),
            hotkey_backend: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
