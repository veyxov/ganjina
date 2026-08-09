<script lang="ts">
  import type { Asset } from './types';
  import { api, monthLabel } from './api';
  import { navigate } from './router.svelte';

  let {
    title,
    assets,
    failedCount = 0,
    showUpload = false,
    onChanged,
  }: {
    title: string;
    assets: Asset[];
    failedCount?: number;
    showUpload?: boolean;
    onChanged: () => void;
  } = $props();

  const groups = $derived.by(() => {
    const out: { label: string; items: Asset[] }[] = [];
    for (const asset of assets) {
      const label = monthLabel(asset.created_at);
      const last = out[out.length - 1];
      if (last && last.label === label) {
        last.items.push(asset);
      } else {
        out.push({ label, items: [asset] });
      }
    }
    return out;
  });

  let uploading = $state(false);

  async function handleUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    uploading = true;
    try {
      await api.upload(input.files);
      onChanged();
    } finally {
      uploading = false;
      input.value = '';
    }
  }

  function open(asset: Asset) {
    navigate(`/assets/${asset.id}`);
  }
</script>

<header class="page-header">
  <h1>{title}</h1>
  <span class="count">
    {assets.length} item{assets.length === 1 ? '' : 's'}
    {#if failedCount > 0}
      &middot; <span class="attention">{failedCount} need{failedCount === 1 ? 's' : ''} attention</span>
    {/if}
  </span>
</header>

{#if showUpload}
  <label class="dropzone">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
      <path d="M12 16V4M12 4l-4 4M12 4l4 4" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M4 16v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span>{uploading ? 'Uploading…' : 'Click to upload an image'}</span>
    <input type="file" accept="image/*,video/*" onchange={handleUpload} disabled={uploading} />
  </label>
{/if}

{#if groups.length === 0}
  <p class="empty">Nothing here yet.</p>
{:else}
  {#each groups as group (group.label)}
    <h2 class="month-label">{group.label}</h2>
    <div class="grid">
      {#each group.items as asset (asset.id)}
        <button class="tile" onclick={() => open(asset)}>
          <img
            src="/blobs/{asset.thumbnail_hash ?? asset.hash}"
            alt={asset.original_filename}
            loading="lazy"
          />
          <span class="name">{asset.original_filename}</span>
        </button>
      {/each}
    </div>
  {/each}
{/if}

<style>
  .page-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-5);
  }
  h1 {
    font-size: 1.05rem;
    font-weight: 600;
    letter-spacing: 0.02em;
  }
  .count {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .attention {
    color: var(--accent);
  }

  .dropzone {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-4);
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--muted);
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
    margin-bottom: var(--space-5);
  }
  .dropzone:hover {
    background: var(--surface-hover);
    color: var(--fg);
  }
  .dropzone input {
    display: none;
  }
  .dropzone svg {
    width: 18px;
    height: 18px;
    opacity: 0.8;
    flex: none;
  }

  .month-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--muted);
    margin: var(--space-5) 0 var(--space-3);
  }
  .month-label:first-of-type {
    margin-top: 0;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  .tile {
    position: relative;
    display: block;
    aspect-ratio: 1;
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--surface);
    transition: transform 0.15s ease;
    border: none;
    padding: 0;
    cursor: pointer;
    width: 100%;
  }
  .tile:hover {
    transform: translateY(-2px);
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

  .empty {
    color: var(--muted);
    font-size: 0.9rem;
    text-align: center;
    padding: var(--space-5) 0;
  }
</style>
