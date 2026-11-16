<script lang="ts">
  import { STATUS_COLORS, type RoadmapNode } from './types';

  let { nodes, onselect }: { nodes: RoadmapNode[]; onselect: (path: string) => void } = $props();

  // Nested by path: sort so that every directory is followed by its subtree.
  const ordered = $derived([...nodes].sort((a, b) => a.path.localeCompare(b.path)));
</script>

<ul class="outline">
  {#each ordered as n (n.path)}
    <li style:padding-left="{n.depth * 16}px">
      {#if n.kind === 'dir'}
        <strong>{n.title}</strong>
      {:else}
        <button onclick={() => onselect(n.path)}>
          <span class="dot" style:background={STATUS_COLORS[n.status].fill}></span>{n.title}
          <small>{n.status}</small>
        </button>
      {/if}
    </li>
  {/each}
</ul>

<style>
  .outline { list-style: none; margin: 0; padding: 0; font-size: 12px; }
  li { padding-top: 2px; padding-bottom: 2px; }
  button { all: unset; cursor: pointer; }
  .dot { display: inline-block; width: 9px; height: 9px; border-radius: 50%; margin-right: 6px; }
  small { opacity: 0.6; margin-left: 6px; }
</style>
