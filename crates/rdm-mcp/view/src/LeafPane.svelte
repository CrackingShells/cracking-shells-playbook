<script lang="ts">
  let { path, markdown, loading, status, onread, ontool }: {
    path: string | null;
    markdown: string | null;
    loading: boolean;
    status: string;
    onread: () => void;
    ontool: () => void;
  } = $props();
</script>

<aside class="pane">
  {#if !path}
    <p class="hint">Select a leaf to read it.</p>
  {:else}
    <h4>{path}</h4>
    <div class="row">
      <button onclick={onread}>Read via resources/read</button>
      <button onclick={ontool}>Try tool call</button>
    </div>
    {#if status}<p class="hint">{status}</p>{/if}
    {#if loading}<p class="hint">loading...</p>{:else}<pre>{markdown ?? '(no markdown in the result: use a button)'}</pre>{/if}
  {/if}
</aside>

<style>
  .pane { border-left: 1px solid color-mix(in srgb, currentColor 25%, transparent); padding-left: 12px; min-width: 260px; max-width: 420px; max-height: 70vh; overflow: auto; }
  h4 { margin: 0 0 6px; font-size: 12px; word-break: break-all; }
  .row { display: flex; gap: 6px; margin-bottom: 6px; }
  pre { margin: 0; font-size: 11px; white-space: pre-wrap; word-break: break-word; }
  .hint { opacity: 0.7; font-size: 12px; word-break: break-word; }
</style>
