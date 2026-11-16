<script lang="ts">
  import { onMount } from 'svelte';
  import { app, connect, callAppTool, requestDisplayMode, type HostInfo } from './bridge';
  import { recordFirstPaint, recordFullView, nextFrameMs, type DisplayModeRan } from './probe';
  import Graph from './Graph.svelte';
  import LeafPane from './LeafPane.svelte';
  import Outline from './Outline.svelte';
  import type { RoadmapView } from './types';

  let view = $state.raw<RoadmapView | null>(null);
  let host = $state.raw<HostInfo | null>(null);
  let error = $state<string | null>(null);
  let theme = $state<'light' | 'dark'>('light');
  let selected = $state<string | null>(null);
  let markdown = $state<string | null>(null);
  let loading = $state(false);
  let outline = $state(false);
  let ran = $state<DisplayModeRan | null>(null);
  let tView = 0;
  let probed = false;

  function readTheme(ctx: unknown) {
    const t = (ctx as { theme?: string } | undefined)?.theme;
    if (t === 'light' || t === 'dark') theme = t;
  }

  onMount(() => {
    connect(
      (p) => {
        const sc = p.structuredContent as unknown as RoadmapView | undefined;
        if (sc && Array.isArray(sc.nodes)) view = sc;
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

  // First paint: once the graph is in the DOM, take t_view_ms in the next frame and probe once.
  $effect(() => {
    if (view && host && !probed) {
      probed = true;
      const v = view;
      const h = host;
      void nextFrameMs().then((t) => {
        tView = t;
        return recordFirstPaint(v, h, t);
      }).catch((e) => (error = String(e)));
    }
  });

  async function openLeaf(path: string) {
    selected = path;
    loading = true;
    try {
      const r = await callAppTool<{ path: string; markdown: string }>('leaf_get', { path });
      markdown = r?.markdown ?? null;
    } catch (e) {
      markdown = 'error: ' + String(e);
    } finally {
      loading = false;
    }
  }

  async function fullView() {
    if (!view || !host) return;
    const ctx = app.getHostContext() as { availableDisplayModes?: string[] } | undefined;
    if (ctx?.availableDisplayModes?.includes('fullscreen')) {
      try {
        const mode = await requestDisplayMode('fullscreen');
        ran = mode === 'fullscreen' ? 'fullscreen' : 'inline';
      } catch (e) {
        error = String(e);
        return;
      }
    } else {
      outline = true;
      ran = 'outline';
    }
    await recordFullView(view, host, tView || Date.now(), ran).catch((e) => (error = String(e)));
  }
</script>

<main class:dark={theme === 'dark'}>
  <header>
    <strong>{view?.campaign ?? 'roadmap'}</strong>
    {#if view}<span class="meta">{view.nodes.length} nodes</span>{/if}
    <button onclick={fullView} disabled={!view}>Full view</button>
    {#if ran}<span class="meta">ran: {ran}</span>{/if}
    {#if outline}<button onclick={() => (outline = false)}>Back to graph</button>{/if}
  </header>
  {#if error}<p class="err">{error}</p>{/if}
  {#if !view}
    <p class="meta">{host ? 'waiting for the tool result...' : 'connecting...'}</p>
  {:else if outline}
    <Outline nodes={view.nodes} onselect={openLeaf} />
    {#if selected}<LeafPane path={selected} {markdown} {loading} />{/if}
  {:else}
    <div class="body">
      <Graph nodes={view.nodes} {selected} onselect={openLeaf} />
      <LeafPane path={selected} {markdown} {loading} />
    </div>
  {/if}
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
