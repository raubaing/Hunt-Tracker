<script lang="ts">
  // Barres horizontales (une série) : barres ≤ 20px, bout arrondi côté valeur, valeur au bout,
  // infobulle au survol de la barre.
  export interface Bar {
    label: string;
    value: number;
    tooltip: string[];
  }

  let { bars, format = (v: number) => v.toFixed(2) }: { bars: Bar[]; format?: (v: number) => string } = $props();

  const max = $derived(Math.max(0.0001, ...bars.map((b) => b.value)));
  let hover = $state<number | null>(null);
</script>

<div class="bars">
  {#each bars as b, i (b.label)}
    <div
      class="row"
      role="img"
      aria-label="{b.label} : {format(b.value)}"
      onpointerenter={() => (hover = i)}
      onpointerleave={() => (hover = null)}
    >
      <span class="label" title={b.label}>{b.label}</span>
      <div class="track">
        <div class="bar" class:active={hover === i} style:width="{(b.value / max) * 100}%"></div>
        <span class="value">{format(b.value)}</span>
      </div>
      {#if hover === i}
        <div class="tooltip">
          {#each b.tooltip as line, j (j)}
            {#if j === 0}<strong>{line}</strong>{:else}<span>{line}</span>{/if}
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .bars { display: flex; flex-direction: column; gap: 6px; }
  .row { position: relative; display: grid; grid-template-columns: 110px 1fr; align-items: center; gap: 10px; padding: 3px 0; }
  .label { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .track { display: flex; align-items: center; gap: 8px; min-width: 0; padding-right: 48px; }
  .bar {
    height: 18px; min-width: 2px; background: var(--accent);
    border-radius: 0 4px 4px 0; /* carré à la base, arrondi au bout */
    transition: width 400ms cubic-bezier(0.2, 0.8, 0.2, 1), filter 120ms ease;
  }
  .bar.active { filter: brightness(1.2); }
  .value { font-size: 12px; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .tooltip {
    position: absolute; z-index: 2; left: 120px; top: calc(100% + 2px); pointer-events: none;
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px;
    padding: 6px 10px; font-size: 12px; display: flex; flex-direction: column; gap: 2px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }
  .tooltip span { color: var(--muted); }
</style>
