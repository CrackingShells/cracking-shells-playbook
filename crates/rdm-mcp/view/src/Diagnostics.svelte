<script lang="ts">
  import type { HostInfo } from './bridge';
  import type { RoadmapView } from './types';

  let { view, host, tView, errors, modelContext, callResult, onCall }: {
    view: RoadmapView | null;
    host: HostInfo | null;
    tView: number;
    errors: Record<string, string>;
    modelContext: string;
    callResult: string;
    onCall: () => void;
  } = $props();

  let pre: HTMLPreElement | undefined = $state();
  let copied = $state('');

  const text = $derived(
    [
      `t_server_ms: ${view?.t_server_ms ?? 'n/a'}`,
      `t_view_ms: ${tView || 'n/a'}`,
      `t_view - t_server: ${view && tView ? tView - view.t_server_ms + ' ms' : 'n/a'}`,
      `payload_bytes: ${view?.payload_bytes ?? 'n/a'}`,
      `apps protocol announced: ${host?.protocolAnnounced ?? 'n/a'}`,
      `apps protocol host: ${host?.protocolHost ?? 'n/a'}`,
      `display mode: ${(host?.hostContext as { displayMode?: string } | undefined)?.displayMode ?? 'n/a'}`,
      `update-model-context: ${modelContext}`,
      `roadmap_view call: ${callResult}`,
      `tool errors: ${Object.keys(errors).length ? '' : 'none'}`,
      ...Object.entries(errors).map(([k, v]) => `  ${k}: ${v}`),
      `hostCapabilities: ${JSON.stringify(host?.hostCapabilities ?? null, null, 2)}`,
      `hostContext: ${JSON.stringify(host?.hostContext ?? null, null, 2)}`,
    ].join('\n'),
  );

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      copied = 'copied';
    } catch {
      // Clipboard is often blocked in a sandboxed iframe: select the text for a manual copy.
      if (pre) {
        const r = document.createRange();
        r.selectNodeContents(pre);
        const sel = window.getSelection();
        sel?.removeAllRanges();
        sel?.addRange(r);
      }
      copied = 'clipboard blocked: text selected, press Cmd/Ctrl+C';
    }
  }
</script>

<details open class="diag">
  <summary>Diagnostics</summary>
  <div class="row">
    <button onclick={onCall}>Call roadmap_view</button>
    <button onclick={copy}>Copy</button>
    <span class="meta">{copied}</span>
  </div>
  <pre bind:this={pre}>{text}</pre>
</details>

<style>
  .diag { margin-top: 12px; font-size: 12px; border-top: 1px solid color-mix(in srgb, currentColor 25%, transparent); padding-top: 6px; }
  .row { display: flex; gap: 8px; align-items: center; margin: 6px 0; }
  pre { margin: 0; font-size: 11px; white-space: pre-wrap; word-break: break-word; max-height: 300px; overflow: auto; user-select: text; }
  .meta { opacity: 0.7; }
</style>
