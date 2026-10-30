<script lang="ts">
  // Fenêtre unique : la barre du widget, et le panneau Sessions / Options qui se déplie en dessous.
  // La fenêtre est transparente : pour déplier, on l'agrandit d'un coup (invisible),
  // puis la carte s'anime jusqu'à la taille du panneau. Pour replier, on fait l'inverse.
  import { onMount, tick } from "svelte";
  import { getCurrentWindow, currentMonitor } from "@tauri-apps/api/window";
  import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
  import type { GameDetails, Preferences, Session, Stat, Tab } from "../lib/types";
  import {
    addStat,
    createSession,
    getActiveSession,
    getSettings,
    onActiveSessionChanged,
    onPreferencesChanged,
    onTogglePanel,
    saveWidgetPosition,
    setGameDetails,
    setPreferences,
    validateGame,
  } from "../lib/storage";
  import { detailsOf } from "../lib/hunt";
  import { isStale } from "../lib/sessions";
  import GameDetailsEditor from "../lib/GameDetailsEditor.svelte";
  import { sessionTotals, kda } from "../lib/stats";
  import { dur, sleep } from "../lib/motion";
  import AnimatedNumber from "../lib/AnimatedNumber.svelte";
  import Panel from "../main/Panel.svelte";

  const BAR = { w: 340, h: 80 };
  const DRAWER_H = 118; // tiroir des infos de game, sous la barre
  const PANEL = { w: 920, h: 640 };
  const DRAWER_AUTOCLOSE_MS = 25000;
  const ANIM_MS = 320;

  const win = getCurrentWindow();

  // ---------- Données ----------
  let active = $state<Session | null>(null);
  // Session restée inactive depuis la veille : elle sera remplacée au premier kill, on l'affiche comme « pas de session »
  let now = $state(new Date());
  const stale = $derived(active ? isStale(active, now) : false);
  const current = $derived(
    (!stale && active?.current) || { kills: 0, deaths: 0, assists: 0 },
  );
  const sessionKda = $derived(active ? kda(sessionTotals(active)) : 0);

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

  let prefs = $state<Preferences>({ showDetailsPrompt: true, locked: false });
  const showDetailsPrompt = $derived(prefs.showDetailsPrompt);
  // Attribut de déplacement : présent ("") si déverrouillé, absent (undefined) si verrouillé
  const dragRegion = $derived(prefs.locked ? undefined : "");
  async function loadPreferences() {
    try {
      prefs = (await getSettings()).preferences;
    } catch (e) {
      console.error(e);
    }
  }
  const toggleLock = () => safe(() => setPreferences({ ...prefs, locked: !prefs.locked }));

  // Quand une game est validée (bouton ou raccourci) : flash de la barre + tiroir des infos.
  // Quand elle est rouverte : on referme le tiroir.
  let flash = $state(false);
  const validate = () => safe(validateGame);
  let last = { sessionId: "", games: -1 };
  $effect(() => {
    const id = active?.id ?? "";
    const n = active?.games.length ?? 0;
    if (id === last.sessionId && last.games >= 0) {
      if (n > last.games) {
        flash = false;
        requestAnimationFrame(() => (flash = true));
        setTimeout(() => (flash = false), 700);
        const game = active?.games[n - 1];
        if (game && showDetailsPrompt && !expanded) openDrawer(game.id);
      } else if (n < last.games) {
        closeDrawer();
      }
    } else if (drawerOpen) {
      closeDrawer();
    }
    last = { sessionId: id, games: n };
  });

  // La game suivante commence (un compteur bouge) : le tiroir se referme
  $effect(() => {
    const c = active?.current;
    if (c && c.kills + c.deaths + c.assists > 0 && drawerOpen) closeDrawer();
  });

  // ---------- Tiroir des infos de la dernière game ----------
  let drawerOpen = $state(false);
  let drawerGameId = $state<number | null>(null);
  let drawerTimer: ReturnType<typeof setTimeout> | undefined;
  let resizing = false;
  const drawerGame = $derived(active?.games.find((g) => g.id === drawerGameId) ?? null);

  function armDrawerTimer() {
    clearTimeout(drawerTimer);
    drawerTimer = setTimeout(closeDrawer, DRAWER_AUTOCLOSE_MS);
  }

  // Le tiroir s'ouvre vers le haut quand il n'y a pas la place sous la barre
  let drawerUp = $state(false);

  async function openDrawer(gameId: number) {
    drawerGameId = gameId;
    armDrawerTimer();
    if (drawerOpen || resizing || busy || expanded) return;
    resizing = true;
    try {
      const bar = await barPosition();
      const area = await workArea();
      const up = area !== null && bar.y + BAR.h + DRAWER_H > area.y + area.h;
      animate = false;
      await win.setSize(new LogicalSize(BAR.w, BAR.h + DRAWER_H));
      if (up) {
        // La fenêtre remonte de la hauteur du tiroir ; la barre reste au même endroit à l'écran
        drawerUp = true;
        offset = { x: 0, y: DRAWER_H };
        await tick();
        await win.setPosition(new LogicalPosition(bar.x, bar.y - DRAWER_H));
      }
      await tick();
      await frame();
      animate = true;
      drawerOpen = true;
      await sleep(dur(ANIM_MS));
    } catch (e) {
      console.error(e);
    } finally {
      resizing = false;
    }
  }

  async function closeDrawer() {
    clearTimeout(drawerTimer);
    if (!drawerOpen || resizing) return;
    resizing = true;
    try {
      animate = true;
      drawerOpen = false;
      await sleep(dur(ANIM_MS));
      if (!expanded && !busy) await snapToBar();
    } catch (e) {
      console.error(e);
    } finally {
      resizing = false;
    }
  }

  async function saveDrawerDetails(d: GameDetails) {
    if (!active || drawerGameId === null) return;
    armDrawerTimer();
    const sessionId = active.id;
    await safe(() => setGameDetails(sessionId, drawerGameId!, d));
  }

  // ---------- Pliage / dépliage ----------
  let expanded = $state(false); // état visuel de la carte
  let panelMounted = $state(false); // le panneau reste monté pendant l'animation de repli
  let animate = $state(false); // active les transitions CSS de la carte
  let offset = $state({ x: 0, y: 0 }); // position de la barre dans la fenêtre agrandie
  let busy = false;
  let tab = $state<Tab>("sessions");
  const TABS: Tab[] = ["sessions", "stats", "options"];

  const frame = () => new Promise((r) => requestAnimationFrame(() => r(null)));

  /** Position de la barre à l'écran (coordonnées logiques) */
  async function barPosition() {
    const sf = await win.scaleFactor();
    const pos = (await win.outerPosition()).toLogical(sf);
    return { x: pos.x + offset.x, y: pos.y + offset.y };
  }

  /** Zone utile de l'écran du widget (sans la barre des tâches), en coordonnées logiques */
  async function workArea() {
    const mon = await currentMonitor();
    if (!mon) return null;
    const a = mon.workArea;
    const f = mon.scaleFactor;
    return { x: a.position.x / f, y: a.position.y / f, w: a.size.width / f, h: a.size.height / f };
  }

  /** Ramène la fenêtre à la taille de la barre seule, à la position de la barre */
  async function snapToBar() {
    animate = false;
    if (offset.x !== 0 || offset.y !== 0) {
      const bar = await barPosition();
      offset = { x: 0, y: 0 };
      drawerUp = false;
      await tick();
      await win.setPosition(new LogicalPosition(bar.x, bar.y));
    }
    drawerUp = false;
    await win.setSize(new LogicalSize(BAR.w, BAR.h));
  }

  async function expand() {
    if (busy || expanded) return;
    busy = true;
    // Le tiroir disparaît : ses infos restent modifiables dans le panneau
    clearTimeout(drawerTimer);
    try {
      if (drawerOpen) {
        drawerOpen = false;
        await snapToBar();
      }
      const pos = await barPosition();

      // Si le panneau dépasse de l'écran, on décale la fenêtre ; la barre garde sa place à l'écran
      let x = pos.x;
      let y = pos.y;
      const area = await workArea();
      if (area) {
        x = Math.max(area.x, Math.min(pos.x, area.x + area.w - PANEL.w));
        y = Math.max(area.y, Math.min(pos.y, area.y + area.h - PANEL.h));
      }

      animate = false;
      await win.setSize(new LogicalSize(PANEL.w, PANEL.h));
      if (x !== pos.x || y !== pos.y) {
        offset = { x: pos.x - x, y: pos.y - y };
        await tick();
        await win.setPosition(new LogicalPosition(x, y));
      }
      panelMounted = true;
      await tick();
      await frame();
      await frame();
      animate = true;
      expanded = true;
      await sleep(dur(ANIM_MS));
    } catch (e) {
      console.error(e);
    } finally {
      busy = false;
    }
  }

  async function collapse() {
    if (busy || !expanded) return;
    busy = true;
    try {
      animate = true;
      expanded = false;
      await sleep(dur(ANIM_MS));

      const sf = await win.scaleFactor();
      const pos = (await win.outerPosition()).toLogical(sf);
      animate = false;
      panelMounted = false;
      if (offset.x !== 0 || offset.y !== 0) {
        const target = new LogicalPosition(pos.x + offset.x, pos.y + offset.y);
        offset = { x: 0, y: 0 };
        await tick();
        await win.setPosition(target);
      }
      await win.setSize(new LogicalSize(BAR.w, BAR.h));
    } catch (e) {
      console.error(e);
    } finally {
      busy = false;
    }
  }

  const toggle = () => (expanded ? collapse() : expand());

  // Ferme la fenêtre : l'appli s'arrête
  async function quit() {
    // Enregistre tout de suite un déplacement récent avant de fermer
    clearTimeout(saveTimer);
    await pendingSave?.();
    await win.destroy();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && drawerOpen && !expanded) {
      closeDrawer();
      return;
    }
    if (e.key !== "Escape" || !expanded) return;
    // Laisse d'abord la main aux champs qui utilisent Échap (saisie d'un raccourci, édition d'une game)
    setTimeout(() => {
      if (!e.defaultPrevented) collapse();
    });
  }

  // ---------- Position mémorisée ----------
  // On enregistre la position de la barre (et non de la fenêtre agrandie), un peu après
  // la fin du déplacement pour ne pas écrire le fichier en continu.
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let pendingSave: (() => Promise<void>) | null = null;

  function onMoved(pos: { x: number; y: number }) {
    clearTimeout(saveTimer);
    pendingSave = async () => {
      pendingSave = null;
      try {
        const sf = await win.scaleFactor();
        await saveWidgetPosition(pos.x + offset.x * sf, pos.y + offset.y * sf);
      } catch (e) {
        console.error(e);
      }
    };
    saveTimer = setTimeout(() => pendingSave?.(), 500);
  }

  onMount(() => {
    refresh();
    const clock = setInterval(() => (now = new Date()), 60_000);
    loadPreferences();
    const offs = [
      onActiveSessionChanged(refresh),
      onPreferencesChanged(loadPreferences),
      onTogglePanel(toggle),
      win.onMoved(({ payload }) => onMoved(payload)),
    ];
    return () => {
      clearTimeout(saveTimer);
      clearTimeout(drawerTimer);
      clearInterval(clock);
      offs.forEach((p) => p.then((f) => f()));
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div
  class="card"
  class:expanded
  class:animate
  class:up={drawerUp && !expanded}
  style:left="{expanded ? 0 : offset.x}px"
  style:top="{expanded ? 0 : offset.y - (drawerOpen && drawerUp ? DRAWER_H : 0)}px"
  style:width="{expanded ? PANEL.w : BAR.w}px"
  style:height="{expanded ? PANEL.h : BAR.h + (drawerOpen ? DRAWER_H : 0)}px"
  style:--anim="{dur(ANIM_MS)}ms"
>
  <div class="header">
    <!-- data-tauri-drag-region : cliquer-glisser sur la barre déplace la fenêtre (sauf si verrouillée) -->
    <div class="bar" class:flash data-tauri-drag-region={dragRegion}>
      <div class="top" data-tauri-drag-region={dragRegion}>
        {#if active && !stale}
          <span class="title" data-tauri-drag-region={dragRegion} title={active.name}>{active.name.replace(/^Session du /, "")}</span>
          <span class="game-no" data-tauri-drag-region={dragRegion} title="Numéro de la game en cours">
            Game {active.games.length + 1}
          </span>
          <span class="kda" data-tauri-drag-region={dragRegion} title="KDA de la session">
            KDA <AnimatedNumber value={sessionKda} decimals={2} />
          </span>
        {:else}
          <button
            class="new"
            title={stale
              ? "La dernière session est inactive depuis plus de 6 h : une nouvelle commencera au premier kill"
              : "Créer la session du jour"}
            onclick={() => safe(createSession)}>＋ Nouvelle session</button
          >
        {/if}
        <div class="win">
          <button
            class="lock"
            class:locked={prefs.locked}
            title={prefs.locked ? "Position verrouillée : clique pour déverrouiller" : "Verrouiller la position du widget"}
            aria-pressed={prefs.locked}
            onclick={toggleLock}
          >
            <svg width="10" height="12" viewBox="0 0 10 12" aria-hidden="true">
              <rect x="1" y="5" width="8" height="6.5" rx="1.2" fill="currentColor" />
              <path
                d={prefs.locked ? "M2.8 5V3.4a2.2 2.2 0 0 1 4.4 0V5" : "M2.8 5V3.4a2.2 2.2 0 0 1 4.3-.7"}
                fill="none"
                stroke="currentColor"
                stroke-width="1.4"
              />
            </svg>
          </button>
          <button
            class="toggle"
            class:open={expanded}
            title={expanded ? "Replier (Échap ou raccourci)" : "Déplier : sessions, options (ou raccourci, F7 par défaut)"}
            onclick={toggle}>⤢</button
          >
          <button title="Quitter Hunt Tracker" onclick={quit}>✕</button>
        </div>
      </div>

      <div class="counters" data-tauri-drag-region={dragRegion}>
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
        <button class="validate" title="Valider la game et passer à la suivante" onclick={validate}>✓</button>
      </div>
    </div>

    {#if panelMounted}
      <nav class="tabs" class:visible={expanded} data-tauri-drag-region={dragRegion}>
        <button class:active={tab === "sessions"} onclick={() => (tab = "sessions")}>Sessions</button>
        <button class:active={tab === "stats"} onclick={() => (tab = "stats")}>Stats</button>
        <button class:active={tab === "options"} onclick={() => (tab = "options")}>Options</button>
        <span class="underline" style:transform="translateX({TABS.indexOf(tab) * 100}px)"></span>
      </nav>
    {/if}
  </div>

  {#if !panelMounted && drawerGame}
    <div class="drawer" class:visible={drawerOpen}>
      <div class="drawer-head">
        <span>Game {active?.games.indexOf(drawerGame)! + 1} — infos <em>(facultatif)</em></span>
        <button class="ok" title="Fermer (Échap)" onclick={closeDrawer}>OK</button>
      </div>
      <GameDetailsEditor details={detailsOf(drawerGame)} onchange={saveDrawerDetails} />
    </div>
  {/if}

  {#if panelMounted}
    <div class="panel" class:visible={expanded}>
      <Panel {tab} />
    </div>
  {/if}
</div>

<style>
  /* ---------- Carte (barre seule ou panneau déplié) ---------- */
  .card {
    position: absolute;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: rgba(22, 19, 15, 0.9);
    border: 1px solid var(--border);
    border-radius: 8px;
    animation: appear 300ms ease-out;
  }
  .card.expanded { background: var(--bg); border-radius: 10px; }
  .card.animate {
    transition:
      left var(--anim) cubic-bezier(0.2, 0.8, 0.2, 1),
      top var(--anim) cubic-bezier(0.2, 0.8, 0.2, 1),
      width var(--anim) cubic-bezier(0.2, 0.8, 0.2, 1),
      height var(--anim) cubic-bezier(0.2, 0.8, 0.2, 1),
      background-color var(--anim) ease,
      border-radius var(--anim) ease;
  }

  .header { display: flex; flex-shrink: 0; border-bottom: 1px solid transparent; }
  .card.expanded .header { border-bottom-color: var(--border); }

  /* ---------- Barre du widget ---------- */
  .bar {
    width: 340px; height: 80px; flex-shrink: 0; box-sizing: border-box;
    display: flex; flex-direction: column; justify-content: center; gap: 6px;
    padding: 6px 10px;
    user-select: none;
  }
  .bar.flash { animation: flash 700ms ease-out; }
  .top { display: flex; align-items: center; gap: 8px; }
  .title { font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .game-no {
    font-size: 11px; color: var(--muted); white-space: nowrap; flex-shrink: 0;
    border: 1px solid var(--border); border-radius: 8px; padding: 0 6px;
  }
  .kda { font-size: 12px; color: var(--muted); white-space: nowrap; margin-left: auto; font-variant-numeric: tabular-nums; }
  .new { padding: 1px 8px; font-size: 12px; margin-right: auto; }
  .win { display: flex; gap: 4px; flex-shrink: 0; }
  .win button { padding: 0 6px; font-size: 12px; }
  .lock { display: inline-flex; align-items: center; color: var(--muted); }
  .lock.locked { color: var(--accent); border-color: var(--accent); }
  .toggle { transition: transform 300ms cubic-bezier(0.2, 0.8, 0.2, 1), border-color 120ms; }
  .toggle.open { transform: rotate(90deg); }
  .toggle.open:active { transform: rotate(90deg) scale(0.95); }
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
  .validate:hover { filter: brightness(1.15); }

  /* ---------- Onglets (à droite de la barre quand c'est déplié) ---------- */
  .tabs {
    position: relative; flex: 1; display: flex; align-items: flex-end; gap: 4px; padding-left: 16px;
    border-left: 1px solid var(--border);
    opacity: 0; transform: translateY(-6px);
    transition: opacity 200ms ease, transform 250ms ease;
  }
  .tabs.visible { opacity: 1; transform: none; transition-delay: 120ms; }
  .tabs button {
    width: 96px; background: none; border: none; border-radius: 0;
    padding: 0 0 12px; color: var(--muted);
  }
  .tabs button:hover, .tabs button.active { color: var(--text); }
  .tabs button:active { transform: none; }
  .underline {
    position: absolute; bottom: -1px; left: 16px; height: 2px; width: 96px;
    background: var(--accent);
    transition: transform 250ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  /* ---------- Tiroir des infos ---------- */
  .drawer {
    height: 118px; flex-shrink: 0; box-sizing: border-box;
    padding: 6px 10px 8px; border-top: 1px solid var(--border);
    display: flex; flex-direction: column; gap: 5px;
    opacity: 0; transform: translateY(-6px);
    transition: opacity 180ms ease, transform 240ms ease;
  }
  .drawer.visible { opacity: 1; transform: none; transition-delay: 100ms; }
  /* Tiroir au-dessus de la barre */
  .card.up { flex-direction: column-reverse; }
  .card.up .drawer { border-top: none; border-bottom: 1px solid var(--border); transform: translateY(6px); }
  .card.up .drawer.visible { transform: none; }
  .drawer-head { display: flex; align-items: center; justify-content: space-between; font-size: 12px; font-weight: 600; }
  .drawer-head em { color: var(--muted); font-weight: 400; font-style: normal; }
  .ok { padding: 0 10px; font-size: 12px; }

  /* ---------- Panneau ---------- */
  .panel {
    flex: 1; min-height: 0;
    opacity: 0; transform: translateY(-8px);
    transition: opacity 180ms ease, transform 260ms ease;
  }
  .panel.visible { opacity: 1; transform: none; transition-delay: 140ms; }

  @keyframes pop {
    0% { color: var(--accent); transform: scale(1.35); }
    100% { color: var(--text); transform: scale(1); }
  }
  @keyframes flash {
    0% { background-color: rgba(200, 16, 46, 0.45); }
    100% { background-color: transparent; }
  }
  @keyframes appear {
    from { opacity: 0; transform: translateY(-6px); }
    to { opacity: 1; transform: none; }
  }
</style>
