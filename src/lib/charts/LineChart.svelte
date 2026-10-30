<script lang="ts">
  // Courbe simple (une série) : ligne 2px, points 8px avec anneau, grille discrète,
  // réticule + infobulle au survol (la colonne la plus proche du curseur).
  export interface LinePoint {
    label: string; // étiquette de l'axe X
    value: number;
    tooltip: string[]; // lignes de l'infobulle (la première en gras)
  }

  let { points, height = 200, format = (v: number) => v.toFixed(2) }: {
    points: LinePoint[];
    height?: number;
    format?: (v: number) => string;
  } = $props();

  let width = $state(600);
  const pad = { top: 12, right: 16, bottom: 26, left: 40 };

  // Graduations « rondes » de 0 au max
  const ticks = $derived.by(() => {
    const max = Math.max(1, ...points.map((p) => p.value));
    const raw = max / 4;
    const mag = 10 ** Math.floor(Math.log10(raw));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
    const n = Math.ceil(max / step);
    return Array.from({ length: n + 1 }, (_, i) => +(i * step).toFixed(6));
  });
  const yMax = $derived(ticks[ticks.length - 1]);
  const innerW = $derived(Math.max(10, width - pad.left - pad.right));
  const innerH = $derived(height - pad.top - pad.bottom);
  const x = (i: number) => pad.left + (points.length === 1 ? innerW / 2 : (i * innerW) / (points.length - 1));
  const y = (v: number) => pad.top + innerH - (v / yMax) * innerH;
  const path = $derived(points.map((p, i) => `${i ? "L" : "M"}${x(i)},${y(p.value)}`).join(""));
  const area = $derived(
    points.length > 1
      ? `${path}L${x(points.length - 1)},${pad.top + innerH}L${x(0)},${pad.top + innerH}Z`
      : "",
  );
  // N'affiche qu'une partie des étiquettes X si elles sont nombreuses
  const labelEvery = $derived(Math.max(1, Math.ceil(points.length / Math.max(1, Math.floor(innerW / 64)))));

  let hover = $state<number | null>(null);

  function onMove(e: PointerEvent) {
    const rect = (e.currentTarget as SVGElement).getBoundingClientRect();
    const px = e.clientX - rect.left;
    if (points.length === 1) return (hover = 0);
    const i = Math.round(((px - pad.left) / innerW) * (points.length - 1));
    hover = Math.max(0, Math.min(points.length - 1, i));
  }
</script>

<div class="chart" bind:clientWidth={width}>
  <svg {width} {height} role="img" aria-label="Graphique" onpointermove={onMove} onpointerleave={() => (hover = null)}>
    {#each ticks as t (t)}
      <line class="grid" x1={pad.left} x2={width - pad.right} y1={y(t)} y2={y(t)} />
      <text class="tick" x={pad.left - 8} y={y(t)} text-anchor="end" dominant-baseline="middle">{format(t)}</text>
    {/each}
    {#each points as p, i (i)}
      {#if i % labelEvery === 0 || i === points.length - 1}
        <text class="tick" x={x(i)} y={height - 8} text-anchor="middle">{p.label}</text>
      {/if}
    {/each}

    {#if area}<path class="area" d={area} />{/if}
    <path class="line" d={path} />

    {#if hover !== null}
      <line class="crosshair" x1={x(hover)} x2={x(hover)} y1={pad.top} y2={pad.top + innerH} />
    {/if}
    {#each points as p, i (i)}
      <circle class="dot" class:active={hover === i} cx={x(i)} cy={y(p.value)} r={hover === i ? 5.5 : 4} />
    {/each}
    <!-- Valeur de la dernière session, en bout de courbe -->
    {#if points.length && hover === null}
      {@const last = points.length - 1}
      <text class="end" x={x(last)} y={y(points[last].value) - 10} text-anchor="end">{format(points[last].value)}</text>
    {/if}
  </svg>

  {#if hover !== null}
    {@const p = points[hover]}
    <div
      class="tooltip"
      style:left="{Math.min(x(hover) + 12, width - 180)}px"
      style:top="{Math.max(0, y(p.value) - 20)}px"
    >
      {#each p.tooltip as line, i (i)}
        {#if i === 0}<strong>{line}</strong>{:else}<span>{line}</span>{/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .chart { position: relative; width: 100%; }
  svg { display: block; overflow: visible; touch-action: none; }
  .grid { stroke: var(--border); stroke-width: 1; }
  .tick { fill: var(--muted); font-size: 11px; font-variant-numeric: tabular-nums; }
  .end { fill: var(--text); font-size: 12px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .line { fill: none; stroke: var(--accent); stroke-width: 2; stroke-linejoin: round; stroke-linecap: round; }
  .area { fill: var(--accent); opacity: 0.1; }
  .dot { fill: var(--accent); stroke: var(--bg); stroke-width: 2; transition: r 120ms ease; }
  .crosshair { stroke: var(--muted); stroke-width: 1; }
  .tooltip {
    position: absolute; pointer-events: none; min-width: 140px; max-width: 170px;
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px;
    padding: 6px 10px; font-size: 12px; display: flex; flex-direction: column; gap: 2px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }
  .tooltip span { color: var(--muted); }
</style>
