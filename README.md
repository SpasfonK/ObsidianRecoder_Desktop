# ObsidianRECORDER Desktop — Itération 1 (Gauntlet Loop)

Portage Windows 11/10 du projet Android **ObsidianRECORDER** : un
enregistreur de mémos vocaux local qui transformera la voix en notes
Markdown exploitables dans un Vault Obsidian. Cette itération pose le
**socle audio**, le **raccourci clavier global** et le **pipeline CI/CD**,
conformément au cahier des charges fourni.

## 1. Stack retenue : Tauri v2 + Rust (Option A)

- **Empreinte mémoire minimale** — exigence explicite du cahier des charges
  pour un outil destiné à tourner en permanence en arrière-plan. Un webview
  Tauri (WebView2, déjà présent sur Windows 11/10) pèse une fraction d'un
  runtime Electron embarqué.
- **Accès bas niveau direct à WASAPI** via `cpal`, sans repasser par une
  couche managée (.NET) ni par du binding NDK.
- **Écosystème Rust natif pour la suite** : `whisper-rs` et les bindings
  Vosk existent nativement en Rust pour l'Itération 2, sans FFI
  supplémentaire.
- **Raccourcis globaux natifs** (`RegisterHotKey`) via la crate
  `global-hotkey`, maintenue par l'équipe Tauri.
- Binaire final compact (LTO + `strip` en profil release, voir
  `src-tauri/Cargo.toml`).

## 2. Ce qui est livré dans cette itération

| Exigence du cahier des charges | Statut |
|---|---|
| Capture audio bas niveau (WASAPI, mono 16 kHz PCM 16 bits) | ✅ `audio::capture` + `audio::dsp` |
| Sélection du périphérique d'entrée | ✅ `list_input_devices` |
| Raccourci clavier global (bascule démarrage/arrêt) | ✅ `Ctrl+Shift+O` par défaut, `hotkey::*` |
| Pipeline CI/CD GitHub Actions → artefact `.exe` | ✅ `.github/workflows/build-windows.yml` |
| Tests : capture simulée → WAV valide | ✅ `tests/audio_pipeline_test.rs` |
| Tests : hotkeys sans collision | ✅ `tests/hotkey_registry_test.rs` |
| STT hors ligne (Vosk/whisper.cpp) | ⏭️ Itération 2 |
| Écriture Markdown/Frontmatter dans le Vault | ⏭️ Itération 3 |
| Fluent Design, tray, overlay, packaging final | ⏭️ Itération 4 |

En attendant l'Itération 3, chaque bascule du raccourci global (ou clic
dans l'interface) écrit un `.wav` horodaté dans
`Documents\ObsidianRECORDER\` — voir `src-tauri/src/paths.rs`.

## 3. Arborescence

```
obsidianrecorder-desktop/
├── package.json, vite.config.js, index.html   # frontend (vanilla + Vite)
├── src/                                        # frontend : main.js, style.css
├── src-tauri/
│   ├── Cargo.toml, build.rs, tauri.conf.json
│   ├── capabilities/default.json               # ACL Tauri v2
│   ├── icons/                                   # icônes originales (générées)
│   ├── src/
│   │   ├── main.rs, lib.rs                      # assemblage Tauri + raccourci par défaut
│   │   ├── app_state.rs, commands.rs, paths.rs
│   │   ├── audio/     (capture.rs, dsp.rs, wav_writer.rs, error.rs)
│   │   └── hotkey/    (binding.rs, registry.rs, os_backend.rs, error.rs)
│   └── tests/
│       ├── audio_pipeline_test.rs               # Gauntlet Test #1
│       └── hotkey_registry_test.rs              # Gauntlet Test #2
└── .github/workflows/build-windows.yml
```

## 4. Décisions d'architecture notables

- **`cpal::Stream` n'est pas `Send`** (contrainte COM/WASAPI sous Windows) :
  chaque enregistrement tourne sur un thread dédié qui crée, garde en vie
  puis détruit le flux ; seuls des types `Send` (canaux `mpsc`, `PathBuf`)
  traversent la frontière entre threads. Voir la note en tête de
  `audio/capture.rs`.
- **`global-hotkey` exige une boucle d'événements Win32 active sur son
  thread de création** : le gestionnaire natif (`OsHotkeyBackend`) est donc
  initialisé dans le hook `setup` de Tauri (thread principal), tandis que
  toute la logique de détection de collision (`HotkeyRegistry`) est pure et
  ne dépend d'aucune API système — c'est elle qui est couverte par les
  tests unitaires.
- **Ré-échantillonnage** : interpolation linéaire simple (suffisante pour
  de la voix à 16 kHz) ; remplaçable par un algorithme sinc en Itération 2
  sans changer l'interface publique de `audio::dsp`.
- **Frontend volontairement minimal** (pas de framework) : il ne fait
  qu'appeler les commandes Tauri (`list_input_devices`,
  `start_recording`, `stop_recording`, `is_recording`,
  `default_output_path`) ; toute la logique métier vit côté Rust et est
  donc testée par `cargo test`.

## 5. Construire le projet

**Prérequis (Windows 11/10) :** Rust stable (`rustup`), Node.js 20+,
[WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)
(préinstallé sur Windows 11 et la plupart des Windows 10 à jour).

```bash
npm install
cargo install tauri-cli --version "^2.0.0"
cargo tauri dev      # mode développement
cargo tauri build    # installeur NSIS + binaire, dans src-tauri/target/release/
```

Tests uniquement (aucun frontend requis) :

```bash
cd src-tauri
cargo test --all
```

### CI/CD

Sur chaque push/PR vers `main`, `.github/workflows/build-windows.yml` :
1. exécute `cargo test --all` sur `windows-latest` (le build s'arrête net
   en cas d'échec — zéro régression, méthode Gauntlet Loop) ;
2. si les tests passent, compile l'application et publie deux artefacts
   téléchargeables dans l'onglet **Actions** : l'installateur NSIS et le
   binaire portable.

## 6. Limites connues et points de vigilance

- **Aucun `Cargo.lock`/`package-lock.json` n'est fourni** : je n'ai pas
  d'accès réseau ni de toolchain Rust dans mon environnement d'exécution
  pour les générer moi-même. Le workflow utilise donc `npm install` (pas
  `npm ci`) et laisse Cargo résoudre les versions à chaque run. Après le
  premier build CI réussi, committez le `package-lock.json` généré et
  envisagez de committer `Cargo.lock` (recommandé pour un binaire
  applicatif) pour des builds reproductibles.
- **Vérification** : je n'ai ni Rust/Cargo, ni machine Windows, ni accès
  réseau dans mon bac à sable — je n'ai donc pas pu compiler ni exécuter ce
  code moi-même. Chaque API de `cpal`, `hound`, `global-hotkey` et Tauri
  v2 utilisée ici a été vérifiée contre leur documentation actuelle avant
  d'être écrite, mais **le run GitHub Actions de votre dépôt est la
  véritable étape de vérification** de ce jalon. S'il signale une erreur
  de compilation, dites-le-moi avec le message d'erreur : je corrige
  immédiatement, dans l'esprit Gauntlet Loop (étape → vérification
  critique → correction).
- Le mapping touche→`Code` de `hotkey::os_backend` couvre lettres,
  chiffres, F1-F12 et les touches de navigation/édition courantes —
  suffisant pour les raccourcis du cahier des charges, à étendre si besoin.
- `is_recording()` reflète l'état « un `stop()` reste à appeler », pas un
  contrôle actif de santé du thread : une erreur matérielle en cours
  d'enregistrement est journalisée (`eprintln!`) mais ne fait pas
  automatiquement basculer l'état — un vrai reporting d'erreur vers
  l'interface est prévu avec le tray system de l'Itération 4.

## 7. Prochaines étapes (Itérations 2 à 4)

2. Intégration Vosk/whisper.cpp (streaming + résultats partiels).
3. Sélecteur de Vault Obsidian, templating Markdown/Frontmatter, écriture
   atomique, modes Append/New File.
4. Fluent Design (Mica/Acrylic), dashboard, mini-overlay avec vu-mètre,
   menu System Tray, installateur final.
