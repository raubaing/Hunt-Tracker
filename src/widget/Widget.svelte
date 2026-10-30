<script lang="ts">
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  async function openMain() {
    const main = await WebviewWindow.getByLabel("main");
    await main?.show();
    await main?.setFocus();
  }
</script>

<!-- data-tauri-drag-region : cliquer-glisser sur le widget le déplace -->
<div class="card" data-tauri-drag-region>
  <span class="title" data-tauri-drag-region>Hunt Tracker</span>
  <button title="Ouvrir la fenêtre principale" onclick={openMain}>⤢</button>
</div>

<style>
  .card {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: rgba(22, 19, 15, 0.85);
    border: 1px solid var(--border);
    border-radius: 8px;
    user-select: none;
  }
  .title { color: var(--muted); font-size: 13px; }
  button { padding: 2px 8px; }
</style>
