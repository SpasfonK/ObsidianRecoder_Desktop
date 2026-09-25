// Masque la console au lancement en mode release sous Windows (l'utilisateur
// final ne doit pas voir de fenêtre de terminal derrière l'application), tout
// en la conservant en mode debug pour suivre les journaux du socle audio et
// hotkey (`println!`/`eprintln!`). Sans effet sur les autres plateformes.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    obsidianrecorder_lib::run();
}
