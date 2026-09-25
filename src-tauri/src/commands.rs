//! Commandes Tauri exposées au frontend via `invoke(...)`.
//!
//! Les erreurs métier (`AudioError`) sont converties en `String` à la
//! frontière IPC : c'est le moyen le plus simple et le plus robuste de les
//! faire transiter vers JavaScript sans avoir à dupliquer toute la
//! hiérarchie d'erreurs côté frontend.

use tauri::State;

use crate::app_state::AppState;
use crate::audio::{self, RecordingSummary};
use crate::paths;

type CmdResult<T> = Result<T, String>;

#[derive(Debug, serde::Serialize)]
pub struct RecordingSummaryDto {
    pub path: String,
    pub duration_seconds: f64,
}

impl From<RecordingSummary> for RecordingSummaryDto {
    fn from(summary: RecordingSummary) -> Self {
        Self {
            path: summary.path.display().to_string(),
            duration_seconds: summary.duration.as_secs_f64(),
        }
    }
}

/// Liste les périphériques d'entrée audio disponibles, pour le sélecteur de
/// microphone de l'interface.
#[tauri::command]
pub fn list_input_devices() -> CmdResult<Vec<String>> {
    audio::list_input_devices().map_err(|e| e.to_string())
}

/// Indique si un enregistrement est actuellement en cours.
#[tauri::command]
pub fn is_recording(state: State<'_, AppState>) -> CmdResult<bool> {
    let capture = state
        .audio
        .lock()
        .map_err(|_| "état audio verrouillé de façon incohérente".to_string())?;
    Ok(capture.is_recording())
}

/// Calcule un chemin de sortie `.wav` horodaté par défaut, dans le dossier
/// de secours `Documents\ObsidianRECORDER` (avant sélection du Vault
/// Obsidian en Itération 3), et crée ce dossier s'il n'existe pas encore.
#[tauri::command]
pub fn default_output_path() -> CmdResult<String> {
    let dir = paths::fallback_recordings_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir
        .join(paths::timestamped_wav_filename())
        .display()
        .to_string())
}

/// Démarre un enregistrement vers `output_path`, sur le périphérique
/// `device` (nom exact renvoyé par [`list_input_devices`]), ou le
/// périphérique par défaut si `device` est `None`.
#[tauri::command]
pub fn start_recording(
    state: State<'_, AppState>,
    device: Option<String>,
    output_path: String,
) -> CmdResult<()> {
    let mut capture = state
        .audio
        .lock()
        .map_err(|_| "état audio verrouillé de façon incohérente".to_string())?;
    capture
        .start(device, std::path::PathBuf::from(output_path))
        .map_err(|e| e.to_string())
}

/// Arrête l'enregistrement en cours et retourne le résumé du fichier WAV
/// finalisé.
#[tauri::command]
pub fn stop_recording(state: State<'_, AppState>) -> CmdResult<RecordingSummaryDto> {
    let mut capture = state
        .audio
        .lock()
        .map_err(|_| "état audio verrouillé de façon incohérente".to_string())?;
    capture
        .stop()
        .map(RecordingSummaryDto::from)
        .map_err(|e| e.to_string())
}
