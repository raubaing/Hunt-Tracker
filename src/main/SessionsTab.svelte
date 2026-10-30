<script lang="ts">
  // Onglet Sessions : liste à gauche, détail de la session sélectionnée à droite
  import { onMount } from "svelte";
  import { flip } from "svelte/animate";
  import { fly, slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import AnimatedNumber from "../lib/AnimatedNumber.svelte";
  import { dur } from "../lib/motion";
  import type { Game, GameDetails, Session } from "../lib/types";
  import { detailsOf, mapShort, teamLabel } from "../lib/hunt";
  import GameDetailsEditor from "../lib/GameDetailsEditor.svelte";
  import {
    listSessions,
    createSession,
    deleteSession,
    getActiveSession,
    setActiveSession,
    validateGame,
    unvalidateGame,
    updateGame,
    deleteGame,
    setGameDetails,
    onSessionsChanged,
    onActiveSessionChanged,
  } from "../lib/storage";
  import { formatStart } from "../lib/sessions";
  import { sessionTotals, kda, formatKda } from "../lib/stats";

  let sessions = $state<Session[]>([]);
  let activeId = $state<string | null>(null);
  let selectedId = $state<string | null>(null);
  let confirmDeleteId = $state<string | null>(null);
  let error = $state<string | null>(null);

  // Correction d'une game validée
  let editing = $state<{ id: number; kills: number; deaths: number; assists: number; details: GameDetails } | null>(null);
  let confirmGameId = $state<number | null>(null);

  const selected = $derived(sessions.find((s) => s.id === selectedId) ?? null);
  const selectedTotals = $derived(selected ? sessionTotals(selected) : null);
  const isActive = $derived(selected !== null && selected.id === activeId);

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
      const s = await createSession();
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

  function startEdit(g: Game) {
    editing = { id: g.id, kills: g.kills, deaths: g.deaths, assists: g.assists, details: detailsOf(g) };
  }

  const clampCount = (n: number) => (Number.isFinite(n) ? Math.max(0, Math.min(999, Math.round(n))) : 0);

  const saveEdit = () =>
    run(async () => {
      if (!selected || !editing) return;
      const e = editing;
      await updateGame(selected.id, e.id, {
        kills: clampCount(e.kills),
        deaths: clampCount(e.deaths),
        assists: clampCount(e.assists),
      });
      await setGameDetails(selected.id, e.id, e.details);
      editing = null;
    });

  function onEditKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") saveEdit();
    if (e.key === "Escape") {
      e.preventDefault(); // n'replie pas le panneau
      editing = null;
    }
  }

  function askDeleteGame(id: number) {
    confirmGameId = id;
    setTimeout(() => {
      if (confirmGameId === id) confirmGameId = null;
    }, 4000);
  }

  const removeGame = (id: number) =>
    run(async () => {
      if (!selected) return;
      confirmGameId = null;
      await deleteGame(selected.id, id);
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
          <li
            animate:flip={{ duration: dur(250) }}
            in:slide={{ duration: dur(220), easing: cubicOut }}
            out:slide={{ duration: dur(180) }}
          >
            <button class="item" class:selected={s.id === selectedId} onclick={() => (selectedId = s.id)}>
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
      {#key selected.id}
      <div in:fly={{ x: 16, duration: dur(220), easing: cubicOut }}>
      <header>
        <div>
          <h1>{selected.name}</h1>
          <p class="sub">Commencée le {formatStart(selected.startedAt)}</p>
        </div>
        <div class="actions">
          {#if isActive}
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
        <div><span>Games</span><strong><AnimatedNumber value={selectedTotals.games} /></strong></div>
        <div><span>Kills</span><strong><AnimatedNumber value={selectedTotals.kills} /></strong></div>
        <div><span>Assists</span><strong><AnimatedNumber value={selectedTotals.assists} /></strong></div>
        <div><span>Morts</span><strong><AnimatedNumber value={selectedTotals.deaths} /></strong></div>
        <div><span>KDA</span><strong><AnimatedNumber value={kda(selectedTotals)} decimals={2} /></strong></div>
      </div>

      {#if selected.games.length === 0 && !isActive}
        <p class="empty">Aucune game enregistrée dans cette session.</p>
      {:else}
        <table>
          <thead>
            <tr><th>#</th><th>K</th><th>A</th><th>D</th><th>KDA</th><th>Infos</th><th></th></tr>
          </thead>
          <tbody>
            {#each selected.games as g, i (g.id)}
              <tr class="game" in:fly={{ y: -8, duration: dur(260), easing: cubicOut }} out:fly={{ x: -16, duration: dur(180) }}>
                <td>{i + 1}</td>
                {#if editing?.id === g.id}
                  <td><input type="number" min="0" bind:value={editing.kills} onkeydown={onEditKeydown} aria-label="Kills" /></td>
                  <td><input type="number" min="0" bind:value={editing.assists} onkeydown={onEditKeydown} aria-label="Assists" /></td>
                  <td><input type="number" min="0" bind:value={editing.deaths} onkeydown={onEditKeydown} aria-label="Morts" /></td>
                  <td>{formatKda(kda({ kills: clampCount(editing.kills), deaths: clampCount(editing.deaths), assists: clampCount(editing.assists) }))}</td>
                  <td class="infos-edit">
                    <GameDetailsEditor details={editing.details} onchange={(d) => editing && (editing.details = d)} />
                  </td>
                  <td class="row-actions visible">
                    <button class="primary small" onclick={saveEdit}>Enregistrer</button>
                    <button class="small" onclick={() => (editing = null)}>Annuler</button>
                  </td>
                {:else}
                  <td>{g.kills}</td><td>{g.assists}</td><td>{g.deaths}</td>
                  <td>{formatKda(kda(g))}</td>
                  <td>
                    <div class="infos">
                      {#if g.teamSize}<span class="chip">{teamLabel(g.teamSize)}</span>{/if}
                      {#if g.map}<span class="chip" title={g.map}>{mapShort(g.map)}</span>{/if}
                      {#if g.extracted === true}<span class="chip good">Extrait</span>{/if}
                      {#if g.extracted === false}<span class="chip bad">Pas extrait</span>{/if}
                      {#if g.bounty === true}<span class="chip good">Bounty</span>{/if}
                      {#if g.bounty === false}<span class="chip">Sans bounty</span>{/if}
                      {#if g.mmrStars}<span class="chip stars" title="{g.mmrStars} étoiles MMR">★{g.mmrStars}</span>{/if}
                    </div>
                  </td>
                  <td class="row-actions" class:visible={confirmGameId === g.id}>
                    {#if confirmGameId === g.id}
                      <button class="danger small" onclick={() => removeGame(g.id)}>Confirmer la suppression</button>
                    {:else}
                      <button class="icon" title="Modifier cette game" onclick={() => startEdit(g)}>✎</button>
                      <button class="icon" title="Supprimer cette game" onclick={() => askDeleteGame(g.id)}>🗑</button>
                    {/if}
                  </td>
                {/if}
              </tr>
            {/each}
            {#if isActive}
              <tr class="current">
                <td>En cours</td>
                <td>{selected.current.kills}</td><td>{selected.current.assists}</td><td>{selected.current.deaths}</td>
                <td>{formatKda(kda(selected.current))}</td>
                <td></td>
                <td></td>
              </tr>
            {/if}
          </tbody>
        </table>
        {#if isActive}
          <div class="current-actions">
            <button class="small" onclick={() => run(validateGame)}>✓ Valider la game en cours</button>
            {#if selected.games.length > 0}
              <button
                class="small"
                title="La dernière game validée redevient la game en cours (Ctrl + raccourci de validation)"
                onclick={() => run(unvalidateGame)}>↶ Rouvrir la dernière game</button
              >
            {/if}
          </div>
        {/if}
      {/if}
      </div>
      {/key}
    {:else}
      <div class="placeholder">
        <p>Crée une session pour commencer à suivre tes games.</p>
        <button class="primary" onclick={create}>＋ Nouvelle session</button>
      </div>
    {/if}
  </section>
</div>

<style>
  .layout { display: grid; grid-template-columns: 260px 1fr; height: 100%; min-height: 0; }
  aside { background: var(--panel); border-right: 1px solid var(--border); padding: 16px; overflow-y: auto; }
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
  tr.current td { color: var(--accent); font-weight: 600; }
  td:first-child { white-space: nowrap; }
  /* Colonnes chiffrées serrées, la colonne Infos prend la place restante */
  table th, table td { width: 1%; white-space: nowrap; padding-right: 16px; }
  table th:nth-child(6), table td:nth-child(6) { width: auto; white-space: normal; }
  .infos { display: flex; flex-wrap: wrap; gap: 4px; }
  .infos-edit { min-width: 300px; }
  .chip {
    font-size: 11px; padding: 1px 6px; border-radius: 10px;
    border: 1px solid var(--border); color: var(--muted); white-space: nowrap;
  }
  .chip.good { color: #8fc98f; border-color: #3d5a3d; }
  .chip.bad { color: #d98a8a; border-color: #5a3434; }
  .chip.stars { color: #e0b54a; border-color: #5a4a26; }
  .row-actions { text-align: right; white-space: nowrap; width: 1%; }
  .row-actions button { margin-left: 6px; font-weight: 400; color: var(--text); }
  .row-actions:not(.visible) button { opacity: 0; transition: opacity 120ms ease; }
  tr.game:hover .row-actions button, .row-actions button:focus-visible { opacity: 1; }
  .row-actions .small { padding: 3px 10px; font-size: 13px; }
  .current-actions { display: flex; gap: 8px; margin-top: 12px; }
  .current-actions .small { padding: 4px 12px; font-size: 13px; }
  .icon { padding: 2px 8px; font-size: 13px; }
  td input {
    width: 48px; padding: 2px 4px; font: inherit; color: var(--text);
    background: var(--bg); border: 1px solid var(--accent); border-radius: 4px;
  }
  .empty { color: var(--muted); }
  .placeholder { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--muted); }
  .primary { background: var(--accent); border-color: var(--accent); padding: 8px 14px; font-weight: 600; }
  .primary:hover { filter: brightness(1.1); }
  .danger { background: var(--accent); border-color: var(--accent); animation: pulse-border 1s ease-in-out infinite; }
  .stats div { transition: border-color 200ms ease; }
  .stats div:hover { border-color: var(--muted); }
  .error { animation: shake 300ms ease; }
  @keyframes shake {
    0%, 100% { transform: none; }
    25% { transform: translateX(-4px); }
    75% { transform: translateX(4px); }
  }
  .error { background: rgba(200, 16, 46, 0.15); border: 1px solid var(--accent); padding: 8px 12px; border-radius: 4px; }
</style>
