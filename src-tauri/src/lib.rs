//! Bibliothèque cœur d'ObsidianRECORDER Desktop.
//!
//! Le point d'entrée binaire (`main.rs`) ne fait qu'appeler [`run`]. Cette
//! séparation lib/bin permet aux tests d'intégration (`tests/`) d'importer
//! directement les modules métier (`audio`, `hotkey`) sans dépendre du
//! runtime Tauri.

pub mod app_state;
pub mod audio;
pub mod commands;
pub mod hotkey;
pub mod paths;

use tauri::Manager;

use app_state::AppState;
use hotkey::{HotkeyAction, HotkeyBinding, HotkeyError};

/// Raccourci global par défaut : bascule démarrage/arrêt de l'enregistrement
/// depuis n'importe quelle application (cahier des charges, section
/// « Expérience Utilisateur Desktop »).
const DEFAULT_TOGGLE_HOTKEY: &str = "Ctrl+Shift+O";

/// Bascule l'enregistrement en cours : démarre si aucun enregistrement
/// n'était actif, arrête et finalise le WAV sinon. Appelé depuis le thread
/// d'écoute du raccourci global (voir [`setup_global_hotkey`]).
fn toggle_recording(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<AppState>();
    let mut capture = match state.audio.lock() {
        Ok(guard) => guard,
        Err(_) => {
            eprintln!("[obsidianrecorder] état audio verrouillé de façon incohérente");
            return;
        }
    };

    if capture.is_recording() {
        match capture.stop() {
            Ok(summary) => println!(
                "[obsidianrecorder] enregistrement terminé : {} ({:.1} s)",
                summary.path.display(),
                summary.duration.as_secs_f64()
            ),
            Err(err) => {
                eprintln!("[obsidianrecorder] échec à l'arrêt de l'enregistrement : {err}")
            }
        }
        return;
    }

    let dir = paths::fallback_recordings_dir();
    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!(
            "[obsidianrecorder] impossible de créer le dossier {} : {err}",
            dir.display()
        );
        return;
    }
    let output_path = dir.join(paths::timestamped_wav_filename());
    match capture.start(None, output_path.clone()) {
        Ok(()) => println!(
            "[obsidianrecorder] enregistrement démarré : {}",
            output_path.display()
        ),
        Err(err) => {
            eprintln!("[obsidianrecorder] échec au démarrage de l'enregistrement : {err}")
        }
    }
}

/// Initialise le raccourci global par défaut et le thread qui écoute ses
/// événements.
///
/// Doit être appelé depuis le hook `setup` de Tauri, sur le thread
/// principal : c'est celui qui fait tourner la boucle d'événements Win32
/// requise par `global-hotkey` pour recevoir les événements `WM_HOTKEY`.
fn setup_global_hotkey(app: &tauri::App) -> Result<(), HotkeyError> {
    let state = app.state::<AppState>();
    let binding = HotkeyBinding::parse(DEFAULT_TOGGLE_HOTKEY)
        .expect("le raccourci par défaut doit être une expression valide");

    let mut backend = hotkey::OsHotkeyBackend::new()?;
    backend.register(&binding)?;

    {
        let mut registry = state
            .hotkeys
            .lock()
            .expect("le registre de raccourcis ne doit pas être empoisonné au démarrage");
        registry.register(binding, HotkeyAction::ToggleRecording)?;
    }

    *state
        .hotkey_backend
        .lock()
        .expect("le verrou du backend de raccourcis ne doit pas être empoisonné au démarrage") =
        Some(backend);

    let app_handle = app.handle().clone();
    std::thread::Builder::new()
        .name("obsidianrecorder-hotkey-listener".into())
        .spawn(move || loop {
            match global_hotkey::GlobalHotKeyEvent::receiver().recv() {
                Ok(event) if event.state == global_hotkey::HotKeyState::Released => {
                    toggle_recording(&app_handle);
                }
                Ok(_) => {}
                // Le canal ne se ferme que si le processus se termine :
                // on arrête simplement la boucle d'écoute.
                Err(_) => break,
            }
        })
        .expect("le thread d'écoute des raccourcis doit pouvoir démarrer");

    Ok(())
}

/// Construit et lance l'application Tauri. Point d'entrée unique appelé par
/// `main.rs`.
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_input_devices,
            commands::is_recording,
            commands::default_output_path,
            commands::start_recording,
            commands::stop_recording,
        ])
        .setup(|app| {
            if let Err(err) = setup_global_hotkey(app) {
                // Un échec d'enregistrement du raccourci (par exemple parce
                // qu'une autre application l'utilise déjà) ne doit pas
                // empêcher l'application de démarrer : l'enregistrement
                // reste possible depuis l'interface.
                eprintln!(
                    "[obsidianrecorder] le raccourci global par défaut ({DEFAULT_TOGGLE_HOTKEY}) n'a pas pu être enregistré : {err}"
                );
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erreur au lancement d'ObsidianRECORDER Desktop");
}
