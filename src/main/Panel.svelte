<script lang="ts">
  // Contenu du panneau déplié : l'onglet actif, avec une transition à chaque changement
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import SessionsTab from "./SessionsTab.svelte";
  import OptionsTab from "./OptionsTab.svelte";
  import { dur } from "../lib/motion";
  import type { Tab } from "../lib/types";

  let { tab }: { tab: Tab } = $props();
</script>

<div class="content">
  {#key tab}
    <div class="pane" in:fly={{ y: 10, duration: dur(220), easing: cubicOut }}>
      {#if tab === "sessions"}
        <SessionsTab />
      {:else}
        <OptionsTab />
      {/if}
    </div>
  {/key}
</div>

<style>
  .content { height: 100%; display: grid; min-height: 0; }
  .pane { grid-area: 1 / 1; min-height: 0; display: flex; flex-direction: column; }
  .pane > :global(*) { flex: 1; min-height: 0; }
</style>
