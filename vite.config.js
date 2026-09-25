import { defineConfig } from "vite";

// Configuration minimale recommandée par Tauri v2 pour un frontend vanilla :
// port fixe pour `tauri dev`, pas de vidage automatique de la console pour
// ne rien perdre des journaux du socle audio/hotkey (`println!`/`eprintln!`
// côté Rust apparaissent dans un terminal séparé, mais les erreurs JS
// restent visibles ici).
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
