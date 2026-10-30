<script lang="ts">
  import { onMount } from "svelte";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { Session, Stat } from "../lib/types";
  import { addStat, createSession, getActiveSession, onActiveSessionChanged, validateGame } from "../lib/storage";
  import { sessionTotals, kda, formatKda } from "../lib/stats";

  let active = $state<Session | null>(null);
  const current = $derived(active?.current ?? { kills: 0, deaths: 0, assists: 0 });
  const sessionKda = $derived(active ? formatKda(kda(sessionTotals(active))) : "–");

  const stats: { key: Stat; label: string; title: string }[] = [
    { key: "kills", label: "K", title: "Kills" },
    { key: "assists", label: "A", title: "Assists" },
    { key: "deaths", label: "D", title: "Morts" },
  ];

  async function refresh() {
    try {
      active = await getActiveSession();
    } catch (e) {
      console.error(e);
    }
  }

  async function safe(action: () => Promise<unknown>) {
    try {
      await action();
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
  <div class="top" data-tauri-drag-region>
    {#if active}
      <span class="title" data-tauri-drag-region>{active.name}</span>
      <span class="kda" data-tauri-drag-region title="KDA de la session">KDA {sessionKda}</span>
    {:else}
      <button class="new" title="Créer la session du jour" onclick={() => safe(createSession)}>＋ Nouvelle session</button>
    {/if}
    <div class="win">
      <button title="Ouvrir la fenêtre principale" onclick={openMain}>⤢</button>
      <button title="Quitter Hunt Tracker" onclick={quit}>✕</button>
    </div>
  </div>

  <div class="counters" data-tauri-drag-region>
    {#each stats as s (s.key)}
      <div class="stat" title={s.title}>
        <button class="pm" onclick={() => safe(() => addStat(s.key, -1))} disabled={current[s.key] === 0}>−</button>
        <span class="label">{s.label}</span>
        {#key current[s.key]}
          <strong class="value">{current[s.key]}</strong>
        {/key}
        <button class="pm" onclick={() => safe(() => addStat(s.key, 1))}>+</button>
      </div>
    {/each}
    <button class="validate" title="Valider la game et passer à la suivante" onclick={() => safe(validateGame)}>✓</button>
  </div>
</div>

<style>
  .card {
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 6px;
    padding: 6px 10px;
    background: rgba(22, 19, 15, 0.88);
    border: 1px solid var(--border);
    border-radius: 8px;
    user-select: none;
  }
  .top { display: flex; align-items: center; gap: 8px; }
  .title { font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .kda { font-size: 12px; color: var(--muted); white-space: nowrap; margin-left: auto; font-variant-numeric: tabular-nums; }
  .new { padding: 1px 8px; font-size: 12px; margin-right: auto; }
  .win { display: flex; gap: 4px; flex-shrink: 0; }
  .win button { padding: 0 6px; font-size: 12px; }
  .counters { display: flex; align-items: center; gap: 6px; }
  .stat {
    display: flex; align-items: center; gap: 3px;
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px; padding: 2px;
  }
  .label { color: var(--muted); font-size: 11px; font-weight: 600; margin-left: 2px; }
  .value {
    min-width: 20px; text-align: center; font-size: 16px; font-variant-numeric: tabular-nums;
    animation: pop 0.35s ease-out;
  }
  .pm { width: 20px; height: 22px; padding: 0; line-height: 1; border-color: transparent; background: transparent; }
  .pm:hover:not(:disabled) { border-color: var(--accent); }
  .pm:disabled { opacity: 0.3; cursor: default; }
  .validate {
    margin-left: auto; height: 28px; width: 32px; padding: 0;
    background: var(--accent); border-color: var(--accent); font-weight: 700;
  }
  @keyframes pop {
    0% { color: var(--accent); transform: scale(1.35); }
    100% { color: var(--text); transform: scale(1); }
  }
</style>
