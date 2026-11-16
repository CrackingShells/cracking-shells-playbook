<script lang="ts">
  import { onMount } from 'svelte';
  import { app, connect, callAppTool, requestDisplayMode, readResourceText, onToolError, sendModelContext, sendChatMessage, type HostInfo } from './bridge';
  import { buildProbe, probeLine } from './probe';
  import { ingest } from './ingest';
  import Graph from './Graph.svelte';
  import LeafPane from './LeafPane.svelte';
  import Outline from './Outline.svelte';
  import Diagnostics from './Diagnostics.svelte';
  import type { RoadmapNode, RoadmapView, Stamps } from './types';

  let view = $state.raw<RoadmapView | null>(null);
  let host = $state.raw<HostInfo | null>(null);
  let nodes = $state.raw<RoadmapNode[]>([]);
  let error = $state<string | null>(null);
  let theme = $state<'light' | 'dark'>('light');
  let selected = $state<string | null>(null);
  let markdown = $state<string | null>(null);
  let leafStatus = $state('');
  let leafRead = $state.raw<unknown>(null);
  let outline = $state(false);
  let ran = $state<string | null>(null);
  let errors = $state<Record<string, string>>({});
  let modelContext = $state('not attempted');
  let messageStatus = $state('not attempted');
  let stamps = $state<Stamps>({
    t_result_ms: 0, t_first_data_ms: 0, t_first_paint_ms: 0, t_complete_ms: 0, bytes_received: 0, nodes_received: 0,
  });
  let started = false;
  onToolError((tool, text) => (errors[tool] = text));

  function readTheme(ctx: unknown) {
    const t = (ctx as { theme?: string } | undefined)?.theme;
    if (t === 'light' || t === 'dark') theme = t;
  }

  function displayMode(): string {
    return ran ?? (host?.hostContext as { displayMode?: string } | undefined)?.displayMode ?? 'inline';
  }

  async function run(sc: RoadmapView) {
    try {
      await ingest(sc, stamps, (n) => (nodes = n));
    } catch (e) {
      error = String(e);
      return;
    }
    // Complete: ONE ui/message probe, plus ui/update-model-context.
    const h = host ?? (await waitHost());
    const probe = buildProbe(sc, stamps, h, displayMode(), leafRead);
    sendModelContext(probe).then((r) => (modelContext = r));
    sendChatMessage(probeLine(probe)).then((r) => (messageStatus = r));
  }

  function waitHost(): Promise<HostInfo> {
    return new Promise((resolve) => {
      const t = setInterval(() => {
        if (host) {
          clearInterval(t);
          resolve(host);
        }
      }, 20);
    });
  }

  onMount(() => {
    connect(
      (p) => {
        const sc = p.structuredContent as unknown as RoadmapView | undefined;
        if (!sc || !sc.mode || started) return;
        started = true;
        stamps.t_result_ms = Date.now();
        view = sc;
        void run(sc);
      },
      readTheme,
    ).then(
      (h) => {
        host = h;
        readTheme(h.hostContext);
      },
      (e) => (error = String(e)),
    );
  });

  function openLeaf(path: string) {
    selected = path;
    leafStatus = '';
    markdown = nodes.find((n) => n.path === path)?.markdown ?? null;
  }

  async function readLeaf() {
    if (!selected) return;
    const uri = 'rdm://leaf/' + selected;
    const t0 = Date.now();
    try {
      markdown = await readResourceText(uri);
      const ms = Date.now() - t0;
      leafStatus = `resources/read ok in ${ms} ms`;
      leafRead = { path: selected, ok: true, ms };
    } catch (e) {
      leafStatus = 'resources/read error: ' + String(e);
      leafRead = { path: selected, ok: false, error: String(e) };
    }
  }

  async function tryTool() {
    if (!selected) return;
    try {
      const r = await callAppTool<{ markdown: string }>('leaf_get', { path: selected });
      markdown = r?.markdown ?? null;
      leafStatus = 'tool call ok';
    } catch (e) {
      leafStatus = 'tool call error: ' + String(e);
    }
  }

  async function fullView() {
    const ctx = app.getHostContext() as { availableDisplayModes?: string[] } | undefined;
    if (ctx?.availableDisplayModes?.includes('fullscreen')) {
      try {
        const mode = await requestDisplayMode('fullscreen');
        ran = mode === 'fullscreen' ? 'fullscreen' : 'inline';
      } catch (e) {
        error = String(e);
      }
    } else {
      outline = true;
      ran = 'outline';
    }
  }
</script>

<main class:dark={theme === 'dark'}>
  <header>
    <strong>roadmap {view ? `(${view.mode} x${view.scale})` : ''}</strong>
    {#if view}<span class="meta">{nodes.length}/{view.node_count} nodes</span>{/if}
    <button onclick={fullView} disabled={!view}>Full view</button>
    {#if ran}<span class="meta">ran: {ran}</span>{/if}
    {#if outline}<button onclick={() => (outline = false)}>Back to graph</button>{/if}
  </header>
  {#if error}<p class="err">{error}</p>{/if}
  {#if !view}
    <p class="meta">{host ? 'waiting for the tool result...' : 'connecting...'}</p>
  {:else if outline}
    <Outline {nodes} onselect={openLeaf} />
    {#if selected}<LeafPane path={selected} {markdown} loading={false} status={leafStatus} onread={readLeaf} ontool={tryTool} />{/if}
  {:else}
    <div class="body">
      <Graph {nodes} {selected} onselect={openLeaf} />
      <LeafPane path={selected} {markdown} loading={false} status={leafStatus} onread={readLeaf} ontool={tryTool} />
    </div>
  {/if}
  <Diagnostics {view} {host} {stamps} {errors} {modelContext} {messageStatus} {displayMode} />
</main>

<style>
  :global(html), :global(body) { margin: 0; }
  :global(body) { font-family: system-ui, sans-serif; }
  main { padding: 10px; color: #111; background: #fff; min-height: 100vh; box-sizing: border-box; }
  main.dark { color: #eee; background: #1b1b1f; }
  header { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; font-size: 13px; }
  .body { display: flex; gap: 12px; align-items: flex-start; }
  .meta { opacity: 0.6; font-size: 12px; }
  .err { color: #dc2626; font-size: 12px; }
</style>
