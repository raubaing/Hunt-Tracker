<script lang="ts">
  // Saisie de tags (coéquipiers) : Entrée ou virgule pour ajouter, × ou Retour arrière pour retirer.
  // Les noms déjà utilisés dans d'autres sessions sont proposés en suggestion.
  import { fly } from "svelte/transition";
  import { dur } from "./motion";

  let {
    tags,
    suggestions = [],
    placeholder = "Ajouter…",
    onchange,
  }: { tags: string[]; suggestions?: string[]; placeholder?: string; onchange: (tags: string[]) => void } = $props();

  let draft = $state("");
  const listId = `tags-${Math.random().toString(36).slice(2)}`;
  const available = $derived(suggestions.filter((s) => !tags.some((t) => t.toLowerCase() === s.toLowerCase())));

  function add(raw: string) {
    const name = raw.trim().replace(/\s+/g, " ");
    draft = "";
    if (!name || tags.some((t) => t.toLowerCase() === name.toLowerCase())) return;
    onchange([...tags, name]);
  }

  const remove = (tag: string) => onchange(tags.filter((t) => t !== tag));

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      add(draft);
    } else if (e.key === "Backspace" && draft === "" && tags.length) {
      remove(tags[tags.length - 1]);
    } else if (e.key === "Escape" && draft) {
      e.preventDefault(); // ne replie pas le panneau
      draft = "";
    }
  }

  // Choix dans la liste de suggestions : l'événement « input » arrive avec la valeur complète
  function onInput() {
    if (available.some((s) => s === draft)) add(draft);
  }
</script>

<div class="tags">
  {#each tags as tag (tag)}
    <span class="tag" in:fly={{ y: -4, duration: dur(160) }}>
      {tag}
      <button title="Retirer {tag}" aria-label="Retirer {tag}" onclick={() => remove(tag)}>×</button>
    </span>
  {/each}
  <input
    list={listId}
    bind:value={draft}
    onkeydown={onKeydown}
    oninput={onInput}
    onblur={() => draft && add(draft)}
    placeholder={tags.length ? "" : placeholder}
    maxlength="30"
  />
  <datalist id={listId}>
    {#each available as s (s)}<option value={s}></option>{/each}
  </datalist>
</div>

<style>
  .tags {
    display: flex; flex-wrap: wrap; align-items: center; gap: 6px;
    padding: 5px 8px; min-height: 34px; box-sizing: border-box;
    background: var(--panel); border: 1px solid var(--border); border-radius: 6px;
    transition: border-color 150ms ease;
  }
  .tags:focus-within { border-color: var(--accent); }
  .tag {
    display: inline-flex; align-items: center; gap: 2px;
    padding: 1px 4px 1px 9px; border-radius: 12px; font-size: 13px;
    background: rgba(200, 16, 46, 0.18); border: 1px solid rgba(200, 16, 46, 0.5);
  }
  .tag button { padding: 0 4px; border: none; background: none; color: var(--muted); font-size: 14px; line-height: 1; }
  .tag button:hover { color: var(--text); }
  input {
    flex: 1; min-width: 120px; border: none; outline: none; background: none;
    font: inherit; font-size: 14px; color: var(--text);
  }
  input::placeholder { color: var(--muted); opacity: 0.7; }
  /* Flèche de la liste de suggestions masquée : elles s'affichent en tapant */
  input::-webkit-calendar-picker-indicator { display: none !important; }
</style>
