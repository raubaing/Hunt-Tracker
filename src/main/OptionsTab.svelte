<script lang="ts">
  // Onglet Options : configuration des raccourcis clavier globaux
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { dur } from "../lib/motion";
  import { updates, checkForUpdates, installUpdate } from "../lib/updates.svelte";
  import type { Preferences, Shortcuts } from "../lib/types";
  import { getSettings, setShortcuts, pauseShortcuts, setPreferences, exportCsv, dataFolderPath, openDataFolder, onPreferencesChanged } from "../lib/storage";
  import { eventToAccelerator, formatAccelerator } from "../lib/shortcuts";

  const DEFAULTS: Shortcuts = { kill: "F8", assist: "F9", death: "F10", validate: "F11", toggle: "F7" };

  // inverse : ce que fait Ctrl + le raccourci (null = rien)
  const rows: { key: keyof Shortcuts; label: string; inverse: string | null }[] = [
    { key: "kill", label: "+1 Kill", inverse: "−1" },
    { key: "assist", label: "+1 Assist", inverse: "−1" },
    { key: "death", label: "+1 Mort", inverse: "−1" },
    { key: "validate", label: "Valider la game", inverse: "rouvrir la dernière" },
    { key: "toggle", label: "Plier / déplier", inverse: null },
  ];

  /** Raccourci inverse : Ctrl + le raccourci (géré automatiquement côté Rust) */
  function ctrlShortcut(accel: string): string | null {
    if (!accel) return null;
    if (/(^|\+)(ctrl|control)(\+|$)/i.test(accel)) return null;
    return formatAccelerator(`Ctrl+${accel}`);
  }

  let shortcuts = $state<Shortcuts>({ ...DEFAULTS });
  let preferences = $state<Preferences>({ showDetailsPrompt: true, locked: false });
  let capturing = $state<keyof Shortcuts | null>(null);
  let error = $state<string | null>(null);
  let saved = $state(false);

  async function load() {
    try {
      const settings = await getSettings();
      shortcuts = settings.shortcuts;
      preferences = settings.preferences;
      error = settings.shortcutError;
    } catch (e) {
      error = String(e);
    }
  }

  async function save(next: Shortcuts) {
    error = null;
    saved = false;
    try {
      await setShortcuts(next);
      shortcuts = next;
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      error = String(e);
      await load();
    }
  }

  async function startCapture(key: keyof Shortcuts) {
    if (capturing) await stopCapture();
    capturing = key;
    // Sinon appuyer sur F8 ici compterait un kill
    await pauseShortcuts(true).catch(() => {});
  }

  async function stopCapture() {
    capturing = null;
    await pauseShortcuts(false).catch((e) => (error = String(e)));
  }

  async function onKeydown(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.code === "Escape") {
      await stopCapture();
      return;
    }
    const accel = eventToAccelerator(e);
    if (!accel) return; // seulement des modificateurs pour l'instant
    const key = capturing;
    capturing = null;
    await save({ ...shortcuts, [key]: accel });
  }

  async function savePreferences(next: Preferences) {
    error = null;
    try {
      await setPreferences(next);
      preferences = next;
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      error = String(e);
    }
  }

  // ---------- Données ----------
  let dataMessage = $state<string | null>(null);
  let folder = $state("");

  async function openFolder() {
    error = null;
    try {
      await openDataFolder();
    } catch (e) {
      error = String(e);
    }
  }
  let exporting = $state(false);

  async function doExport() {
    error = null;
    dataMessage = null;
    exporting = true;
    try {
      const path = await exportCsv();
      if (path) dataMessage = `Exporté : ${path}`;
    } catch (e) {
      error = String(e);
    } finally {
      exporting = false;
    }
  }

  const clear = (key: keyof Shortcuts) => save({ ...shortcuts, [key]: "" });
  const reset = () => save({ ...DEFAULTS });

  onMount(() => {
    load();
    if (!updates.current) checkForUpdates(true);
    dataFolderPath().then((p) => (folder = p)).catch(() => {});
    // Le cadenas du widget modifie aussi les préférences : on reste synchronisé
    const off = onPreferencesChanged(async () => {
      try {
        preferences = (await getSettings()).preferences;
      } catch {}
    });
    return () => {
      off.then((f) => f());
      if (capturing) pauseShortcuts(false).catch(() => {});
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="options">
<div class="inner">
  <h1>Options</h1>

  <h2>Raccourcis clavier</h2>
  <p class="help">
    Ils fonctionnent même quand le jeu est au premier plan. Clique sur un raccourci puis appuie sur la touche
    (ou combinaison) voulue. <kbd>Échap</kbd> annule.
  </p>
  <p class="help">
    <kbd>Ctrl</kbd> + le raccourci fait l'inverse : il retire 1 à un compteur (<kbd>Ctrl</kbd> + <kbd>F8</kbd> retire
    un kill) ou rouvre la dernière game validée (<kbd>Ctrl</kbd> + <kbd>F11</kbd>).
  </p>

  {#if error}
    <p class="error" transition:fade={{ duration: dur(200) }}>{error}</p>
  {/if}

  <table>
    <thead>
      <tr><th></th><th>Raccourci</th><th>Avec Ctrl</th><th></th></tr>
    </thead>
    <tbody>
      {#each rows as row (row.key)}
        <tr>
          <td class="label">{row.label}</td>
          <td>
            <button
              class="key"
              class:capturing={capturing === row.key}
              class:none={!shortcuts[row.key]}
              onclick={() => (capturing === row.key ? stopCapture() : startCapture(row.key))}
            >
              {capturing === row.key ? "Appuie sur une touche…" : formatAccelerator(shortcuts[row.key])}
            </button>
          </td>
          <td class="minus">
            {#if shortcuts[row.key] && row.inverse}
              {@const ctrl = ctrlShortcut(shortcuts[row.key])}
              {#if ctrl}
                {ctrl} <span class="what">{row.inverse}</span>
              {:else}
                <span title="Le raccourci contient déjà Ctrl : pas de raccourci inverse">—</span>
              {/if}
            {/if}
          </td>
          <td>
            {#if shortcuts[row.key]}
              <button class="link" title="Désactiver ce raccourci" onclick={() => clear(row.key)}>Désactiver</button>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  <div class="footer">
    <button onclick={reset}>Rétablir les valeurs par défaut</button>
    {#if saved}<span class="saved" transition:fade={{ duration: dur(200) }}>✓ Enregistré</span>{/if}
  </div>

  <p class="help">
    Les touches choisies sont réservées par Hunt Tracker tant qu'il tourne : elles n'arrivent plus au jeu.
    Évite donc des touches que tu utilises dans Hunt.
  </p>

  <h2 class="section">Widget</h2>
  <label class="check">
    <input
      type="checkbox"
      checked={preferences.locked}
      onchange={(e) => savePreferences({ ...preferences, locked: e.currentTarget.checked })}
    />
    Verrouiller la position du widget (aussi avec le cadenas dans la barre)
  </label>

  <h2 class="section">Après chaque game</h2>
  <label class="check">
    <input
      type="checkbox"
      checked={preferences.showDetailsPrompt}
      onchange={(e) => savePreferences({ ...preferences, showDetailsPrompt: e.currentTarget.checked })}
    />
    Proposer les infos de la game (extraction, bounty, format, map, étoiles) juste après la validation
  </label>
  <p class="help">
    Le tiroir s'ouvre sous le widget et se referme tout seul dès que la game suivante commence. Les infos restent
    modifiables dans l'onglet Sessions (bouton ✎).
  </p>

  <h2 class="section">Mises à jour</h2>
  <div class="update-box">
    <p>
      Version installée : <strong>{updates.current || "…"}</strong>
      {#if updates.status === "uptodate"}<span class="ok">· à jour</span>{/if}
    </p>
    {#if updates.status === "available"}
      <p class="new">Nouvelle version disponible : <strong>{updates.version}</strong></p>
      {#if updates.notes}<pre class="notes">{updates.notes}</pre>{/if}
      <button class="primary-btn" onclick={installUpdate}>Installer et redémarrer</button>
    {:else if updates.status === "downloading" || updates.status === "installing"}
      <p>{updates.status === "downloading" ? "Téléchargement…" : "Installation… l'appli va redémarrer."}</p>
      <div class="bar"><div style:width="{Math.round(updates.progress * 100)}%"></div></div>
    {:else}
      <button onclick={() => checkForUpdates(false)} disabled={updates.status === "checking"}>
        {updates.status === "checking" ? "Recherche…" : "Vérifier les mises à jour"}
      </button>
      {#if updates.status === "error"}
        <p class="err">Impossible de vérifier : {updates.error}</p>
      {/if}
    {/if}
    <p class="help">
      L'appli vérifie toute seule au lancement. Les mises à jour s'installent sans toucher à tes sessions ni à
      tes réglages.
    </p>
  </div>

  <h2 class="section">Données</h2>
  <div class="data-actions">
    <button onclick={doExport} disabled={exporting}>{exporting ? "Export…" : "Exporter en CSV"}</button>
    <button onclick={openFolder}>Ouvrir le dossier des données</button>
  </div>
  {#if dataMessage}
    <p class="data-message" transition:fade={{ duration: dur(200) }}>✓ {dataMessage}</p>
  {/if}
  <p class="help">
    Une ligne par game (toutes les sessions), prête à ouvrir dans Excel : séparateur « ; » et virgule décimale.
  </p>
  {#if folder}
    <p class="help">
      Tes sessions sont enregistrées dans <code>{folder}</code> (un fichier par session). Copie ce dossier pour
      faire une sauvegarde.
    </p>
  {/if}
</div>
</div>

<style>
  /* Le conteneur défilant prend toute la largeur ; seul le contenu est limité */
  .options { padding: 24px 32px; overflow-y: auto; overflow-x: hidden; }
  .inner { max-width: 760px; }
  h1 { margin: 0 0 16px; font-size: 24px; }
  h2 { font-size: 16px; margin: 0 0 4px; }
  .help { color: var(--muted); font-size: 13px; line-height: 1.5; }
  table { border-collapse: collapse; margin: 16px 0; }
  td { padding: 6px 12px 6px 0; }
  th { text-align: left; color: var(--muted); font-size: 12px; font-weight: 600; padding: 0 12px 4px 0; }
  .what { font-family: "Segoe UI", system-ui, sans-serif; font-size: 12px; margin-left: 4px; }
  .minus { white-space: nowrap; min-width: 120px; color: var(--muted); font-family: Consolas, monospace; font-size: 13px; }
  .label { min-width: 140px; }
  .key {
    min-width: 200px; padding: 6px 12px; text-align: left;
    font-family: Consolas, monospace; background: var(--bg);
  }
  .key.none { color: var(--muted); font-style: italic; }
  .key.capturing { border-color: var(--accent); color: var(--accent); animation: pulse-border 1.2s ease-in-out infinite; }
  .link { background: none; border: none; color: var(--muted); text-decoration: underline; padding: 0; }
  .link:hover { color: var(--text); }
  .footer { display: flex; align-items: center; gap: 12px; }
  .footer button { padding: 6px 12px; }
  .saved { color: #7fbf7f; font-size: 13px; }
  .section { margin-top: 28px; }
  .update-box p { margin: 6px 0; font-size: 14px; }
  .update-box button { padding: 6px 12px; margin-top: 4px; }
  .update-box .ok { color: #8fc98f; }
  .update-box .new strong { color: var(--accent); }
  .update-box .err { color: #d98a8a; font-size: 13px; word-break: break-word; }
  .notes {
    white-space: pre-wrap; font: inherit; font-size: 13px; color: var(--muted);
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px; padding: 8px 10px;
    max-height: 140px; overflow-y: auto; margin: 6px 0;
  }
  .primary-btn { background: var(--accent); border-color: var(--accent); color: #fff; font-weight: 600; }
  .bar { height: 6px; background: var(--panel); border-radius: 3px; overflow: hidden; max-width: 320px; }
  .bar div { height: 100%; background: var(--accent); transition: width 200ms ease; }
  .data-actions { display: flex; gap: 8px; margin-top: 8px; }
  .data-actions button { padding: 6px 12px; }
  code { font-family: Consolas, monospace; font-size: 12px; color: var(--text); word-break: break-all; }
  .data-message { color: #8fc98f; font-size: 13px; word-break: break-all; margin: 8px 0 0; }
  .check { display: flex; align-items: center; gap: 8px; cursor: pointer; }
  .check input { accent-color: var(--accent); width: 16px; height: 16px; }
  kbd { border: 1px solid var(--border); border-radius: 3px; padding: 0 4px; font-size: 12px; }
  .error { background: rgba(200, 16, 46, 0.15); border: 1px solid var(--accent); padding: 8px 12px; border-radius: 4px; }
</style>
