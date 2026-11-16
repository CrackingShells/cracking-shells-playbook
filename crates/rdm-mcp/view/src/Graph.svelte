<script lang="ts">
  import { STATUS_COLORS, type RoadmapNode } from './types';

  let { nodes, selected, onselect }: { nodes: RoadmapNode[]; selected: string | null; onselect: (path: string) => void } =
    $props();

  const columns = $derived.by(() => {
    const cols: RoadmapNode[][] = [];
    for (const n of nodes) (cols[n.depth] ??= []).push(n);
    return cols;
  });
</script>

<div class="graph">
  {#each columns as col, depth}
    {#if col}
      <section class="col">
        <h3>depth {depth}</h3>
        {#each col as n (n.path)}
          {#if n.kind === 'dir'}
            <div class="dir" title={n.path}>{n.title}</div>
          {:else}
            <button
              class="card"
              class:selected={selected === n.path}
              style:background={STATUS_COLORS[n.status].fill}
              style:color={STATUS_COLORS[n.status].color}
              title={n.path}
              onclick={() => onselect(n.path)}
            >
              {n.title}
              <small>{n.status}</small>
            </button>
          {/if}
        {/each}
      </section>
    {/if}
  {/each}
</div>

<style>
  .graph { display: flex; gap: 12px; align-items: flex-start; overflow-x: auto; padding-bottom: 8px; }
  .col { display: flex; flex-direction: column; gap: 6px; min-width: 150px; }
  h3 { margin: 0; font-size: 11px; font-weight: 600; opacity: 0.6; text-transform: uppercase; }
  .dir { font-weight: 700; font-size: 12px; padding: 4px 2px; border-bottom: 2px solid currentColor; }
  .card { text-align: left; border: 2px solid transparent; border-radius: 6px; padding: 6px 8px; font: inherit; font-size: 12px; cursor: pointer; display: flex; flex-direction: column; gap: 2px; }
  .card small { opacity: 0.75; font-size: 10px; }
  .card.selected { border-color: #fff; outline: 2px solid #0a84ff; }
</style>
