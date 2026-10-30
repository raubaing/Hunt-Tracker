<script lang="ts">
  // Fenêtre principale : liste des sessions à gauche, détail de la session sélectionnée à droite
  import { onMount } from "svelte";
  import type { Session } from "../lib/types";
  import {
    listSessions,
    deleteSession,
    getActiveSession,
    setActiveSession,
    onSessionsChanged,
    onActiveSessionChanged,
  } from "../lib/storage";
  import { createSession, formatStart } from "../lib/sessions";
  import { totals, kda, formatKda } from "../lib/stats";

  let sessions = $state<Session[]>([]);
  let activeId = $state<string | null>(null);
  let selectedId = $state<string | null>(null);
  let confirmDeleteId = $state<string | null>(null);
  let error = $state<string | null>(null);

  const selected = $derived(sessions.find((s) => s.id === selectedId) ?? null);
  const selectedTotals = $derived(selected ? totals(selected.games) : null);

  async function refresh() {
    try {
      sessions = await listSessions();
      activeId = (await getActiveSession())?.id ?? null;
      if (!selectedId || !sessions.some((s) => s.id === selectedId)) {
        selectedId = activeId ?? sessions[0]?.id ?? null;
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
    } catch (e) {
      error = String(e);
    }
  }

  const create = () =>
    run(async () => {
      const s = await createSession(sessions);
      selectedId = s.id;
    });

  const activate = (id: string) => run(() => setActiveSession(id));

  function askDelete(id: string) {
    confirmDeleteId = id;
    setTimeout(() => {
      if (confirmDeleteId === id) confirmDeleteId = null;
    }, 4000);
  }

  const remove = (id: string) =>
    run(async () => {
      confirmDeleteId = null;
      await deleteSession(id);
      if (selectedId === id) selectedId = null;
    });

  onMount(() => {
    refresh();
    const unlisteners = [onSessionsChanged(refresh), onActiveSessionChanged(refresh)];
    return () => unlisteners.forEach((p) => p.then((off) => off()));
  });
</script>

<div class="layout">
  <aside>
    <button class="primary new" onclick={create}>＋ Nouvelle session</button>

    {#if sessions.length === 0}
      <p class="empty">Aucune session pour l'instant.</p>
    {:else}
      <ul>
        {#each sessions as s (s.id)}
          <li>
            <button
              class="item"
              class:selected={s.id === selectedId}
              onclick={() => (selectedId = s.id)}
            >
              <span class="name">{s.name}</span>
              <span class="meta">
                {#if s.id === activeId}<span class="dot" title="Session en cours"></span>{/if}
                {s.games.length} game{s.games.length > 1 ? "s" : ""}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </aside>

  <section>
    {#if error}
      <p class="error">{error}</p>
    {/if}

    {#if selected && selectedTotals}
      <header>
        <div>
          <h1>{selected.name}</h1>
          <p class="sub">Commencée le {formatStart(selected.startedAt)}</p>
        </div>
        <div class="actions">
          {#if selected.id === activeId}
            <span class="badge">● Session en cours</span>
          {:else}
            <button onclick={() => activate(selected.id)}>Définir comme session en cours</button>
          {/if}
          {#if confirmDeleteId === selected.id}
            <button class="danger" onclick={() => remove(selected.id)}>Confirmer la suppression</button>
          {:else}
            <button onclick={() => askDelete(selected.id)}>Supprimer</button>
          {/if}
        </div>
      </header>

      <div class="stats">
        <div><span>Games</span><strong>{selectedTotals.games}</strong></div>
        <div><span>Kills</span><strong>{selectedTotals.kills}</strong></div>
        <div><span>Morts</span><strong>{selectedTotals.deaths}</strong></div>
        <div><span>Assists</span><strong>{selectedTotals.assists}</strong></div>
        <div><span>KDA</span><strong>{formatKda(kda(selectedTotals))}</strong></div>
      </div>

      {#if selected.games.length === 0}
        <p class="empty">Aucune game enregistrée dans cette session.</p>
      {:else}
        <table>
          <thead>
            <tr><th>#</th><th>K</th><th>D</th><th>A</th><th>KDA</th></tr>
          </thead>
          <tbody>
            {#each selected.games as g (g.id)}
              <tr>
                <td>{g.id}</td><td>{g.kills}</td><td>{g.deaths}</td><td>{g.assists}</td>
                <td>{formatKda(kda(g))}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {:else}
      <div class="placeholder">
        <p>Crée une session pour commencer à suivre tes games.</p>
        <button class="primary" onclick={create}>＋ Nouvelle session</button>
      </div>
    {/if}
  </section>
</div>

<style>
  .layout { display: grid; grid-template-columns: 260px 1fr; height: 100vh; }
  aside {
    background: var(--panel); border-right: 1px solid var(--border);
    padding: 16px; overflow-y: auto;
  }
  section { padding: 24px 32px; overflow-y: auto; }
  ul { list-style: none; margin: 12px 0 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .new { width: 100%; }
  .item {
    width: 100%; text-align: left; padding: 8px 10px; display: flex; flex-direction: column; gap: 2px;
    background: transparent; border-color: transparent;
  }
  .item:hover { background: var(--bg); border-color: var(--border); }
  .item.selected { background: var(--bg); border-color: var(--accent); }
  .name { font-weight: 600; }
  .meta { color: var(--muted); font-size: 12px; display: flex; align-items: center; gap: 6px; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
  header { display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; flex-wrap: wrap; }
  h1 { margin: 0; font-size: 24px; }
  .sub { margin: 4px 0 0; color: var(--muted); }
  .actions { display: flex; gap: 8px; align-items: center; }
  .actions button { padding: 6px 12px; }
  .badge { color: var(--accent); font-weight: 600; }
  .stats { display: grid; grid-template-columns: repeat(5, 1fr); gap: 8px; margin: 24px 0; }
  .stats div {
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px;
    padding: 10px 12px; display: flex; flex-direction: column; gap: 4px;
  }
  .stats span { color: var(--muted); font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; }
  .stats strong { font-size: 22px; font-variant-numeric: tabular-nums; }
  table { width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums; }
  th, td { text-align: left; padding: 8px; border-bottom: 1px solid var(--border); }
  th { color: var(--muted); font-weight: 600; }
  .empty { color: var(--muted); }
  .placeholder { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--muted); }
  .primary { background: var(--accent); border-color: var(--accent); padding: 8px 14px; font-weight: 600; }
  .primary:hover { filter: brightness(1.1); }
  .danger { background: var(--accent); border-color: var(--accent); }
  .error { background: rgba(200, 16, 46, 0.15); border: 1px solid var(--accent); padding: 8px 12px; border-radius: 4px; }
</style>
