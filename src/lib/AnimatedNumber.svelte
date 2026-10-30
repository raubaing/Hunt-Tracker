<script lang="ts">
  // Nombre qui « défile » jusqu'à sa nouvelle valeur
  import { Tween } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { dur } from "./motion";

  let { value, decimals = 0 }: { value: number; decimals?: number } = $props();

  const tween = Tween.of(() => value, { duration: dur(450), easing: cubicOut });
</script>

{tween.current.toFixed(decimals)}
