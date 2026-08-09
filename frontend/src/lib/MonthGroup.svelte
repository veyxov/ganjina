<script lang="ts">
  import { JustifiedLayout } from '@immich/justified-layout-wasm';
  import type { Asset } from './types';
  import { navigate } from './router.svelte';

  let { label, items }: { label: string; items: Asset[] } = $props();

  let containerWidth = $state(0);

  const layout = $derived.by(() => {
    if (containerWidth === 0) return null;
    const ratios = new Float32Array(
      items.map((a) => (a.width && a.height ? a.width / a.height : 1)),
    );
    return new JustifiedLayout(ratios, {
      rowHeight: 200,
      rowWidth: containerWidth,
      spacing: 4,
      heightTolerance: 0.15,
    });
  });

  function open(asset: Asset) {
    navigate(`/assets/${asset.id}`);
  }
</script>

<h2 class="month-label">{label}</h2>
<div class="wall" bind:clientWidth={containerWidth} style:height={layout ? `${layout.containerHeight}px` : undefined}>
  {#if layout}
    {#each items as asset, i (asset.id)}
      {@const pos = layout.getPosition(i)}
      <button
        class="tile"
        style:top="{pos.top}px"
        style:left="{pos.left}px"
        style:width="{pos.width}px"
        style:height="{pos.height}px"
        onclick={() => open(asset)}
      >
        <img src="/blobs/{asset.thumbnail_hash ?? asset.hash}" alt={asset.original_filename} loading="lazy" />
        <span class="name">{asset.original_filename}</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .month-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--muted);
    margin: var(--space-5) 0 var(--space-3);
  }
  .month-label:first-of-type {
    margin-top: 0;
  }

  .wall {
    position: relative;
    margin-bottom: var(--space-4);
  }

  .tile {
    position: absolute;
    display: block;
    border: none;
    padding: 0;
    cursor: pointer;
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--surface);
    transition: transform 0.15s ease;
  }
  .tile:hover {
    transform: scale(1.02);
    z-index: 1;
  }
  .tile img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .tile .name {
    position: absolute;
    inset: auto 0 0 0;
    padding: var(--space-2) var(--space-2) var(--space-1);
    font-size: 0.72rem;
    color: #fff;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.65), transparent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0;
    transition: opacity 0.15s ease;
    text-align: left;
  }
  .tile:hover .name {
    opacity: 1;
  }
</style>
