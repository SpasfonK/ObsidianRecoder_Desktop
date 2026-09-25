import { invoke } from "@tauri-apps/api/core";

const deviceSelect = document.querySelector("#device-select");
const toggleButton = document.querySelector("#toggle-button");
const statusDot = document.querySelector("#status-dot");
const statusLabel = document.querySelector("#status-label");
const lastRecording = document.querySelector("#last-recording");

let recording = false;

async function loadDevices() {
  try {
    const devices = await invoke("list_input_devices");
    deviceSelect.innerHTML = "";
    if (devices.length === 0) {
      const option = document.createElement("option");
      option.textContent = "Aucun microphone détecté";
      deviceSelect.appendChild(option);
      deviceSelect.disabled = true;
      return;
    }
    for (const name of devices) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      deviceSelect.appendChild(option);
    }
  } catch (error) {
    console.error("Impossible de lister les périphériques audio :", error);
  }
}

function setStatus(state, label) {
  statusDot.className = `dot ${state}`;
  statusLabel.textContent = label;
}

async function refreshRecordingState() {
  try {
    recording = await invoke("is_recording");
    if (recording) {
      setStatus("recording", "Enregistrement en cours…");
      toggleButton.textContent = "Arrêter l'enregistrement";
    } else {
      setStatus("idle", "Prêt");
      toggleButton.textContent = "Démarrer l'enregistrement";
    }
  } catch (error) {
    console.error("Impossible de lire l'état d'enregistrement :", error);
  }
}

async function toggleRecording() {
  toggleButton.disabled = true;
  try {
    if (recording) {
      const summary = await invoke("stop_recording");
      lastRecording.textContent = `Dernier enregistrement : ${summary.path} (${summary.duration_seconds.toFixed(1)} s)`;
    } else {
      const device = deviceSelect.value || null;
      // Chemin de secours (Documents\ObsidianRECORDER) tant que la
      // sélection du Vault Obsidian (Itération 3) n'est pas encore
      // disponible dans l'interface.
      const outputPath = await invoke("default_output_path");
      await invoke("start_recording", { device, outputPath });
    }
  } catch (error) {
    console.error("Échec de la bascule d'enregistrement :", error);
    setStatus("error", "Erreur — voir la console");
  } finally {
    toggleButton.disabled = false;
    await refreshRecordingState();
  }
}

toggleButton.addEventListener("click", toggleRecording);

loadDevices();
refreshRecordingState();
// Le raccourci global peut démarrer/arrêter l'enregistrement pendant que
// cette fenêtre n'est pas au premier plan : on resynchronise l'affichage
// périodiquement plutôt que d'exiger un clic pour le voir.
setInterval(refreshRecordingState, 2000);
