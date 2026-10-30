<script lang="ts">
  import { onMount } from "svelte";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { Session } from "../lib/types";
  import { getActiveSession, listSessions, onActiveSessionChanged } from "../lib/storage";
  import { createSession } from "../lib/sessions";

  let active = $state<Session | null>(null);

  async function refresh() {
    try {
      active = await getActiveSession();
    } catch (e) {
      console.error(e);
    }
  }

  async function newSession() {
    try {
      await createSession(await listSessions());
    } catch (e) {
      console.error(e);
    }
  }

  async function openMain() {
    const main = await WebviewWindow.getByLabel("main");
    await main?.show();
    await main?.unminimize();
    await main?.setFocus();
  }

  // Ferme toutes les fenêtres : l'appli s'arrête quand il n'en reste plus
  async function quit() {
    const main = await WebviewWindow.getByLabel("main");
    await main?.destroy();
    await getCurrentWindow().destroy();
  }

  onMount(() => {
    refresh();
    const off = onActiveSessionChanged(refresh);
    return () => off.then((f) => f());
  });
</script>

<!-- data-tauri-drag-region : cliquer-glisser sur le widget le déplace -->
<div class="card" data-tauri-drag-region>
  {#if active}
    <span class="title" data-tauri-drag-region>{active.name}</span>
  {:else}
    <button class="new" title="Créer la session du jour" onclick={newSession}>＋ Nouvelle session</button>
  {/if}
  <div class="actions">
    <button title="Ouvrir la fenêtre principale" onclick={openMain}>⤢</button>
    <button title="Quitter Hunt Tracker" onclick={quit}>✕</button>
  </div>
</div>

<style>
  .card {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 0 12px;
    background: rgba(22, 19, 15, 0.85);
    border: 1px solid var(--border);
    border-radius: 8px;
    user-select: none;
  }
  .title { font-size: 13px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .new { padding: 2px 8px; font-size: 12px; }
  .actions { display: flex; gap: 6px; flex-shrink: 0; }
  button { padding: 2px 8px; }
</style>
