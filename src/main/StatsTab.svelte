<script lang="ts">
  // Onglet Stats : toutes les sessions, avec filtres période / format / map
  import { onMount } from "svelte";
  import type { Session } from "../lib/types";
  import { listSessions, onSessionsChanged } from "../lib/storage";
  import {
    filterGames,
    summarize,
    kdaBySession,
    kdaByMap,
    records,
    sessionDuration,
    formatDuration,
    formatKda,
    type Period,
    type StatsFilter,
  } from "../lib/stats";
  import { MAPS, TEAM_SIZES, mapShort, teamLabel } from "../lib/hunt";
  import AnimatedNumber from "../lib/AnimatedNumber.svelte";
  import LineChart from "../lib/charts/LineChart.svelte";
  import BarChart from "../lib/charts/BarChart.svelte";

  let sessions = $state<Session[]>([]);
  let error = $state<string | null>(null);
  let filter = $state<StatsFilter>({ period: "all", teamSize: null, map: null });

  const PERIODS: { value: Period; label: string }[] = [
    { value: "7d", label: "7 jours" },
    { value: "30d", label: "30 jours" },
    { value: "all", label: "Tout" },
  ];

  const refs = $derived(filterGames(sessions, filter));
  const sum = $derived(summarize(refs));
  const bySession = $derived(kdaBySession(refs));
  const byMap = $derived(kdaByMap(refs));
  const rec = $derived(records(refs));
  // Temps de jeu estimé (début de session → dernière game), uniquement sans filtre format / map
  const playTime = $derived.by(() => {
    if (filter.teamSize !== null || filter.map !== null) return null;
    const ids = new Set(refs.map((r) => r.session.id));
    return sessions.filter((s) => ids.has(s.id)).reduce((t, s) => t + (sessionDuration(s) ?? 0), 0);
  });

  const pct = (n: number, d: number) => (d ? `${Math.round((n / d) * 100)} %` : "–");
  const shortDate = (iso: string) => {
    const d = new Date(iso);
    return `${String(d.getDate()).padStart(2, "0")}/${String(d.getMonth() + 1).padStart(2, "0")}`;
  };
  const plural = (n: number, w: string) => `${n} ${w}${n > 1 ? "s" : ""}`;

  async function refresh() {
    try {
      sessions = await listSessions();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(() => {
    refresh();
    const off = onSessionsChanged(refresh);
    return () => off.then((f) => f());
  });
</script>

<div class="stats-tab">
  <div class="filters">
    <div class="group">
      {#each PERIODS as p (p.value)}
        <button class:on={filter.period === p.value} onclick={() => (filter.period = p.value)}>{p.label}</button>
      {/each}
    </div>
    <div class="group">
      <button class:on={filter.teamSize === null} onclick={() => (filter.teamSize = null)}>Tous formats</button>
      {#each TEAM_SIZES as t (t.value)}
        <button class:on={filter.teamSize === t.value} onclick={() => (filter.teamSize = t.value)}>{t.label}</button>
      {/each}
    </div>
    <div class="group">
      <button class:on={filter.map === null} onclick={() => (filter.map = null)}>Toutes maps</button>
      {#each MAPS as m (m.name)}
        <button class:on={filter.map === m.name} title={m.name} onclick={() => (filter.map = m.name)}>{m.short}</button>
      {/each}
    </div>
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  {#if sum.games === 0}
    <p class="empty">
      Aucune game pour ces filtres.
      {#if filter.teamSize !== null || filter.map !== null}
        Les filtres format et map ne comptent que les games où l'info a été renseignée.
      {/if}
    </p>
  {:else}
    <div class="tiles">
      <div><span>Games</span><strong><AnimatedNumber value={sum.games} /></strong></div>
      <div><span>KDA</span><strong><AnimatedNumber value={sum.kda} decimals={2} /></strong></div>
      <div><span>K/D</span><strong><AnimatedNumber value={sum.kd} decimals={2} /></strong></div>
      <div><span>Kills / game</span><strong><AnimatedNumber value={sum.killsPerGame} decimals={1} /></strong></div>
      <div title="Sur {sum.extractedKnown} game(s) où l'extraction est renseignée">
        <span>Extraction</span><strong>{pct(sum.extracted, sum.extractedKnown)}</strong>
        <em>{sum.extracted} / {sum.extractedKnown}</em>
      </div>
      <div title="Sur {sum.bountyKnown} game(s) où le bounty est renseigné">
        <span>Bounty</span><strong>{pct(sum.bounty, sum.bountyKnown)}</strong>
        <em>{sum.bounty} / {sum.bountyKnown}</em>
      </div>
    </div>
    <p class="totals">
      {sum.kills} kills · {sum.assists} assists · {sum.deaths} morts
      {#if playTime}· environ {formatDuration(playTime)} de jeu{/if}
    </p>

    <div class="grid">
      <section class="card wide">
        <h3>KDA par session</h3>
        {#if bySession.length < 2}
          <p class="empty">Il faut au moins 2 sessions pour tracer l'évolution.</p>
        {:else}
          <LineChart
            points={bySession.map((s) => ({
              label: shortDate(s.session.startedAt),
              value: s.kda,
              tooltip: [
                `KDA ${formatKda(s.kda)}`,
                s.session.name,
                `${plural(s.games, "game")} · ${s.kills} K · ${s.assists} A · ${s.deaths} D`,
              ],
            }))}
          />
        {/if}
      </section>

      <section class="card">
        <h3>KDA par map</h3>
        {#if byMap.length === 0}
          <p class="empty">Renseigne la map de tes games pour voir cette stat.</p>
        {:else}
          <BarChart
            bars={byMap.map((m) => ({
              label: mapShort(m.map),
              value: m.kda,
              tooltip: [`KDA ${formatKda(m.kda)}`, m.map, `${plural(m.games, "game")} · ${m.kills} K · ${m.assists} A · ${m.deaths} D`],
            }))}
          />
        {/if}
      </section>

      <section class="card">
        <h3>Records</h3>
        <ul class="records">
          <li>
            <span>Meilleure game</span>
            {#if rec.bestGame}
              <strong>{plural(rec.bestGame.game.kills, "kill")}</strong>
              <em>
                {rec.bestGame.session.name}{rec.bestGame.game.map ? ` · ${mapShort(rec.bestGame.game.map)}` : ""}{rec
                  .bestGame.game.teamSize
                  ? ` · ${teamLabel(rec.bestGame.game.teamSize)}`
                  : ""}
              </em>
            {:else}<strong>–</strong>{/if}
          </li>
          <li>
            <span>Meilleure session <small>(3 games min.)</small></span>
            {#if rec.bestSession}
              <strong>KDA {formatKda(rec.bestSession.kda)}</strong>
              <em>{rec.bestSession.session.name} · {plural(rec.bestSession.games, "game")}</em>
            {:else}<strong>–</strong>{/if}
          </li>
          <li>
            <span>Plus de kills en une session</span>
            {#if rec.mostKillsSession}
              <strong>{plural(rec.mostKillsSession.kills, "kill")}</strong>
              <em>{rec.mostKillsSession.session.name}</em>
            {:else}<strong>–</strong>{/if}
          </li>
          <li>
            <span>Série sans mourir</span>
            <strong>{plural(rec.longestNoDeathStreak, "game")}</strong>
            <em>d'affilée</em>
          </li>
        </ul>
      </section>
    </div>
  {/if}
</div>

<style>
  .stats-tab { padding: 20px 32px 32px; overflow-y: auto; }
  .filters { display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 20px; }
  .group { display: flex; border: 1px solid var(--border); border-radius: 6px; overflow: hidden; }
  .group button { border: none; border-radius: 0; padding: 5px 11px; font-size: 13px; background: var(--panel); color: var(--muted); }
  .group button + button { border-left: 1px solid var(--border); }
  .group button:hover { color: var(--text); }
  .group button.on { background: var(--accent); color: #fff; }
  .group button:active { transform: none; }

  .tiles { display: grid; grid-template-columns: repeat(6, 1fr); gap: 8px; }
  .tiles div {
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px;
    padding: 10px 12px; display: flex; flex-direction: column; gap: 2px;
  }
  .tiles span { color: var(--muted); font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; }
  .tiles strong { font-size: 22px; font-variant-numeric: tabular-nums; }
  .tiles em { color: var(--muted); font-size: 11px; font-style: normal; }
  .totals { color: var(--muted); font-size: 13px; margin: 10px 0 20px; }

  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .card { background: var(--panel); border: 1px solid var(--border); border-radius: 8px; padding: 14px 16px; min-width: 0; }
  .card.wide { grid-column: 1 / -1; }
  h3 { margin: 0 0 12px; font-size: 14px; }

  .records { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
  .records li { display: grid; grid-template-columns: 1fr auto; column-gap: 12px; }
  .records span { color: var(--muted); font-size: 13px; }
  .records small { font-size: 11px; }
  .records strong { text-align: right; font-variant-numeric: tabular-nums; }
  .records em { grid-column: 1 / -1; color: var(--muted); font-size: 12px; font-style: normal; opacity: 0.8; }

  .empty { color: var(--muted); font-size: 13px; }
  .error { background: rgba(200, 16, 46, 0.15); border: 1px solid var(--accent); padding: 8px 12px; border-radius: 4px; }
</style>
