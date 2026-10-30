<script lang="ts">
  // Message temporaire avec un bouton « Annuler » (remplace les doubles confirmations)
  import { fly } from "svelte/transition";
  import { dur } from "./motion";

  export interface UndoMessage {
    id: number;
    text: string;
    undo: () => Promise<unknown>;
  }

  let { message, onclose }: { message: UndoMessage | null; onclose: () => void } = $props();
  const DURATION = 6000;

  let busy = $state(false);

  $effect(() => {
    if (!message) return;
    busy = false;
    const timer = setTimeout(onclose, DURATION);
    return () => clearTimeout(timer);
  });

  async function undo() {
    if (!message || busy) return;
    busy = true;
    try {
      await message.undo();
    } finally {
      onclose();
    }
  }
</script>

{#if message}
  {#key message.id}
    <div class="toast" role="status" transition:fly={{ y: 16, duration: dur(200) }} style:--duration="{DURATION}ms">
      <span>{message.text}</span>
      <button onclick={undo} disabled={busy}>Annuler</button>
      <button class="close" title="Fermer" aria-label="Fermer" onclick={onclose}>✕</button>
      <span class="progress"></span>
    </div>
  {/key}
{/if}

<style>
  .toast {
    position: absolute; left: 50%; bottom: 20px; transform: translateX(-50%); z-index: 10;
    display: flex; align-items: center; gap: 12px; overflow: hidden;
    padding: 10px 12px 10px 16px; border-radius: 8px;
    background: var(--panel); border: 1px solid var(--border);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5); font-size: 14px; white-space: nowrap;
  }
  button { padding: 4px 12px; font-weight: 600; color: var(--accent); border-color: var(--accent); background: transparent; }
  button:hover { background: rgba(200, 16, 46, 0.15); }
  .close { padding: 2px 6px; font-weight: 400; color: var(--muted); border-color: transparent; }
  .close:hover { background: none; color: var(--text); }
  .progress {
    position: absolute; left: 0; bottom: 0; height: 2px; width: 100%;
    background: var(--accent); transform-origin: left;
    animation: shrink var(--duration) linear forwards;
  }
  @keyframes shrink { to { transform: scaleX(0); } }
</style>
