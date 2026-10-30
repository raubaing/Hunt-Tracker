<script lang="ts">
  // Fenêtre principale : onglets Sessions / Options
  import SessionsTab from "./SessionsTab.svelte";
  import OptionsTab from "./OptionsTab.svelte";

  type Tab = "sessions" | "options";
  let tab = $state<Tab>("sessions");
</script>

<div class="app">
  <nav>
    <span class="brand">Hunt Tracker</span>
    <button class:active={tab === "sessions"} onclick={() => (tab = "sessions")}>Sessions</button>
    <button class:active={tab === "options"} onclick={() => (tab = "options")}>Options</button>
  </nav>
  <div class="content">
    {#if tab === "sessions"}
      <SessionsTab />
    {:else}
      <OptionsTab />
    {/if}
  </div>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  nav {
    display: flex; align-items: center; gap: 4px; padding: 0 16px;
    border-bottom: 1px solid var(--border); background: var(--bg); flex-shrink: 0;
  }
  .brand { font-weight: 700; margin-right: 16px; color: var(--accent); }
  nav button {
    background: none; border: none; border-bottom: 2px solid transparent; border-radius: 0;
    padding: 12px 14px; color: var(--muted);
  }
  nav button:hover { color: var(--text); }
  nav button.active { color: var(--text); border-bottom-color: var(--accent); }
  .content { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .content > :global(*) { flex: 1; min-height: 0; }
</style>
