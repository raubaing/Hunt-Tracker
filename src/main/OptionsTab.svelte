<script lang="ts">
  // Onglet Options : configuration des raccourcis clavier globaux
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { dur } from "../lib/motion";
  import type { Shortcuts } from "../lib/types";
  import { getSettings, setShortcuts, pauseShortcuts } from "../lib/storage";
  import { eventToAccelerator, formatAccelerator } from "../lib/shortcuts";

  const DEFAULTS: Shortcuts = { kill: "F8", assist: "F9", death: "F10", validate: "F11" };

  const rows: { key: keyof Shortcuts; label: string; counter: boolean }[] = [
    { key: "kill", label: "+1 Kill", counter: true },
    { key: "assist", label: "+1 Assist", counter: true },
    { key: "death", label: "+1 Mort", counter: true },
    { key: "validate", label: "Valider la game", counter: false },
  ];

  /** Raccourci inverse : Ctrl + le raccourci (géré automatiquement côté Rust) */
  function ctrlShortcut(accel: string): string | null {
    if (!accel) return null;
    if (/(^|\+)(ctrl|control)(\+|$)/i.test(accel)) return null;
    return formatAccelerator(`Ctrl+${accel}`);
  }

  let shortcuts = $state<Shortcuts>({ ...DEFAULTS });
  let capturing = $state<keyof Shortcuts | null>(null);
  let error = $state<string | null>(null);
  let saved = $state(false);

  async function load() {
    try {
      const settings = await getSettings();
      shortcuts = settings.shortcuts;
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

  const clear = (key: keyof Shortcuts) => save({ ...shortcuts, [key]: "" });
  const reset = () => save({ ...DEFAULTS });

  onMount(() => {
    load();
    return () => {
      if (capturing) pauseShortcuts(false).catch(() => {});
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="options">
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
            {#if shortcuts[row.key]}
              {@const ctrl = ctrlShortcut(shortcuts[row.key])}
              {#if ctrl}
                {ctrl} <span class="what">{row.counter ? "−1" : "rouvrir la dernière"}</span>
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
</div>

<style>
  .options { padding: 24px 32px; max-width: 640px; overflow-y: auto; }
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
  kbd { border: 1px solid var(--border); border-radius: 3px; padding: 0 4px; font-size: 12px; }
  .error { background: rgba(200, 16, 46, 0.15); border: 1px solid var(--accent); padding: 8px 12px; border-radius: 4px; }
</style>
