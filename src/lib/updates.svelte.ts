// Mises à jour automatiques : vérification au lancement, téléchargement, installation, redémarrage.
// L'état est partagé entre le widget (pastille « MAJ ») et l'onglet Options.
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";

type Status = "idle" | "checking" | "uptodate" | "available" | "downloading" | "installing" | "error";

export const updates = $state({
  current: "",
  status: "idle" as Status,
  version: "", // version disponible
  notes: "",
  progress: 0, // 0 → 1 pendant le téléchargement
  error: "",
});

let pending: Update | null = null;

/** Cherche une nouvelle version. En mode silencieux, une erreur réseau n'est pas affichée. */
export async function checkForUpdates(silent = false) {
  if (updates.status === "checking" || updates.status === "downloading" || updates.status === "installing") return;
  if (!updates.current) updates.current = await getVersion().catch(() => "");
  updates.status = "checking";
  updates.error = "";
  try {
    const update = await check();
    pending = update;
    if (update) {
      updates.status = "available";
      updates.version = update.version;
      updates.notes = update.body ?? "";
    } else {
      updates.status = "uptodate";
    }
  } catch (e) {
    updates.status = silent ? "idle" : "error";
    updates.error = String(e);
  }
}

/** Télécharge et installe la mise à jour, puis relance l'appli (les données ne sont pas touchées). */
export async function installUpdate() {
  if (!pending) return;
  updates.status = "downloading";
  updates.progress = 0;
  let total = 0;
  let received = 0;
  try {
    await pending.downloadAndInstall((event) => {
      if (event.event === "Started") total = event.data.contentLength ?? 0;
      if (event.event === "Progress") {
        received += event.data.chunkLength;
        updates.progress = total ? Math.min(1, received / total) : 0;
      }
      if (event.event === "Finished") updates.status = "installing";
    });
    // Sous Windows, l'installeur ferme l'appli lui-même ; ailleurs on relance
    await relaunch();
  } catch (e) {
    updates.status = "error";
    updates.error = String(e);
  }
}
