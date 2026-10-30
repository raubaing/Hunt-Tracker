<script lang="ts">
  // Saisie rapide des infos Hunt d'une game : un clic sur une option la sélectionne,
  // un nouveau clic la désélectionne. Utilisé dans le tiroir du widget et dans le panneau.
  import type { GameDetails, TeamSize } from "./types";
  import { MAPS, STARS, TEAM_SIZES } from "./hunt";

  let { details, onchange }: { details: GameDetails; onchange: (d: GameDetails) => void } = $props();

  function set<K extends keyof GameDetails>(key: K, value: GameDetails[K]) {
    onchange({ ...details, [key]: details[key] === value ? null : value });
  }
</script>

<div class="editor">
  <div class="row">
    <span class="label">Extraction</span>
    <button class:on={details.extracted === true} onclick={() => set("extracted", true)}>Oui</button>
    <button class:on={details.extracted === false} onclick={() => set("extracted", false)}>Non</button>
    <span class="label sep">Bounty</span>
    <button class:on={details.bounty === true} onclick={() => set("bounty", true)}>Oui</button>
    <button class:on={details.bounty === false} onclick={() => set("bounty", false)}>Non</button>
  </div>
  <div class="row">
    {#each TEAM_SIZES as t (t.value)}
      <button class:on={details.teamSize === t.value} onclick={() => set("teamSize", t.value as TeamSize)}>{t.label}</button>
    {/each}
    <span class="stars" title="Étoiles MMR">
      {#each STARS as n (n)}
        <button
          class="star"
          class:on={(details.mmrStars ?? 0) >= n}
          title="{n} étoile{n > 1 ? 's' : ''}"
          onclick={() => set("mmrStars", n)}>★</button
        >
      {/each}
    </span>
  </div>
  <div class="row">
    {#each MAPS as m (m.name)}
      <button class:on={details.map === m.name} title={m.name} onclick={() => set("map", m.name)}>{m.short}</button>
    {/each}
  </div>
</div>

<style>
  .editor { display: flex; flex-direction: column; gap: 5px; }
  .row { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; }
  .label { color: var(--muted); font-size: 11px; margin-right: 2px; }
  .sep { margin-left: 8px; }
  button { padding: 2px 8px; font-size: 12px; background: var(--panel); }
  button.on { background: var(--accent); border-color: var(--accent); color: #fff; }
  .stars { display: inline-flex; margin-left: 8px; }
  .star { padding: 0 2px; font-size: 15px; line-height: 1; background: none; border: none; color: var(--border); }
  .star.on { background: none; color: #e0b54a; }
  .star:hover { color: #e0b54a; }
</style>
