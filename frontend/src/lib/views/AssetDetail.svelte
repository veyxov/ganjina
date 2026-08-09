<script lang="ts">
  import type { Asset, Collection, Owner, PhotoMetadata } from '../types';
  import { api, humanSize } from '../api';
  import { navigate } from '../router.svelte';

  let { id }: { id: string } = $props();

  let asset = $state<Asset | null>(null);
  let metadata = $state<PhotoMetadata | null>(null);
  let prevId = $state<string | null>(null);
  let nextId = $state<string | null>(null);
  let inCollections = $state<Collection[]>([]);
  let allCollections = $state<Collection[]>([]);
  let owners = $state<Owner[]>([]);
  let fullscreen = $state(false);
  let newOwnerName = $state('');

  const availableCollections = $derived(
    allCollections.filter((c) => !inCollections.some((ic) => ic.id === c.id)),
  );

  async function load() {
    fullscreen = false;
    const [a, m, adj, inColl, all, own] = await Promise.all([
      api.getAsset(id),
      api.getMetadata(id),
      api.getAdjacent(id),
      api.collectionsForAsset(id),
      api.listCollections(),
      api.listOwners(),
    ]);
    asset = a;
    metadata = m;
    prevId = adj.prev_id;
    nextId = adj.next_id;
    inCollections = inColl;
    allCollections = all;
    owners = own;
  }

  $effect(() => {
    id;
    load();
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') fullscreen = false;
  }

  async function setOwner(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    await api.setOwner(id, value || null);
    await load();
  }

  async function createOwner(e: Event) {
    e.preventDefault();
    const name = newOwnerName.trim();
    if (!name) return;
    await api.createOwner(name);
    newOwnerName = '';
    owners = await api.listOwners();
  }

  async function addToCollection(e: Event) {
    const collectionId = (e.target as HTMLSelectElement).value;
    if (!collectionId) return;
    await api.addAssetToCollection(collectionId, id);
    await load();
  }

  async function removeFromCollection(collectionId: string) {
    await api.removeAssetFromCollection(collectionId, id);
    await load();
  }

  async function deleteAsset() {
    if (!confirm('Delete this item permanently?')) return;
    await api.deleteAsset(id);
    navigate('/');
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if asset}
  <div class="page-header">
    <a class="back-link" href="/" onclick={(e) => { e.preventDefault(); navigate('/'); }}>&larr; Back</a>
    <span>
      {#if prevId}
        <a class="back-link" href="/assets/{prevId}" onclick={(e) => { e.preventDefault(); navigate(`/assets/${prevId}`); }}>&larr; Prev</a>
      {/if}
      {#if nextId}
        <a class="back-link next" href="/assets/{nextId}" onclick={(e) => { e.preventDefault(); navigate(`/assets/${nextId}`); }}>Next &rarr;</a>
      {/if}
    </span>
  </div>

  <div class="detail">
    <div class="preview" class:fullscreen>
      {#if asset.content_type.startsWith('video/')}
        <video src="/blobs/{asset.hash}" controls>
          <track kind="captions" />
        </video>
      {:else}
        <button class="zoom" onclick={() => (fullscreen = !fullscreen)} title="Click to view fullscreen">
          <img src="/blobs/{asset.hash}" alt={asset.original_filename} />
        </button>
      {/if}
    </div>
    <div class="panel">
      <h2>Info</h2>
      <dl>
        <dt>Name</dt>
        <dd>{asset.original_filename}</dd>
        <dt>Type</dt>
        <dd>{asset.content_type}</dd>
        <dt>Size</dt>
        <dd>{humanSize(asset.size_bytes)}</dd>
        <dt>Uploaded</dt>
        <dd>{new Date(asset.created_at).toLocaleString()}</dd>
        {#if metadata?.taken_at_local}
          <dt>Taken</dt>
          <dd>
            {new Date(metadata.taken_at_local).toLocaleString()}
            {#if metadata.taken_at_offset_minutes !== null}(UTC{metadata.taken_at_offset_minutes >= 0 ? '+' : ''}{metadata.taken_at_offset_minutes}min){/if}
          </dd>
        {/if}
        {#if metadata?.camera}
          <dt>Camera</dt>
          <dd>{metadata.camera}</dd>
        {/if}
        {#if metadata?.gps_lat !== null && metadata?.gps_lat !== undefined}
          <dt>Location</dt>
          <dd>{metadata.gps_lat}, {metadata.gps_lon}</dd>
        {/if}
      </dl>

      <h2>Owner</h2>
      <select value={asset.owner_id ?? ''} onchange={setOwner}>
        <option value="">Unassigned</option>
        {#each owners as o (o.id)}
          <option value={o.id}>{o.name}</option>
        {/each}
      </select>
      <form class="new-item" onsubmit={createOwner}>
        <input type="text" placeholder="New owner" bind:value={newOwnerName} />
        <button type="submit" title="Create">+</button>
      </form>

      <h2>Collections</h2>
      {#if inCollections.length === 0}
        <p class="empty">Not in any collection.</p>
      {:else}
        <ul class="chip-list">
          {#each inCollections as c (c.id)}
            <li class="chip">
              <a href="/collections/{c.id}" onclick={(e) => { e.preventDefault(); navigate(`/collections/${c.id}`); }}>{c.name}</a>
              <button onclick={() => removeFromCollection(c.id)} title="Remove">&times;</button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if availableCollections.length > 0}
        <select value="" onchange={addToCollection}>
          <option value="" disabled>Add to collection…</option>
          {#each availableCollections as c (c.id)}
            <option value={c.id}>{c.name}</option>
          {/each}
        </select>
      {/if}

      <form class="delete-form" onsubmit={(e) => { e.preventDefault(); deleteAsset(); }}>
        <button type="submit" class="delete-btn">Delete</button>
      </form>
    </div>
  </div>
{/if}

<style>
  .page-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }
  .back-link {
    color: var(--muted);
    text-decoration: none;
    font-size: 0.85rem;
  }
  .back-link:hover {
    color: var(--fg);
  }
  .back-link.next {
    margin-left: var(--space-3);
  }

  .detail {
    display: flex;
    gap: var(--space-4);
    align-items: flex-start;
  }
  .preview {
    flex: 1;
    min-width: 0;
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--surface);
    display: flex;
  }
  .preview .zoom {
    width: 100%;
    display: block;
    border: none;
    padding: 0;
    background: none;
    cursor: zoom-in;
  }
  .preview img,
  .preview video {
    width: 100%;
    display: block;
  }
  .preview.fullscreen {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: #000;
    border-radius: 0;
    width: 100vw;
    height: 100vh;
  }
  .preview.fullscreen .zoom {
    width: 100%;
    height: 100%;
    cursor: zoom-out;
  }
  .preview.fullscreen img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .panel {
    flex: none;
    width: 260px;
    font-size: 0.85rem;
  }
  .panel h2 {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    margin: var(--space-4) 0 var(--space-2);
  }
  .panel h2:first-child {
    margin-top: 0;
  }
  .panel dt {
    color: var(--muted);
  }
  .panel dd {
    margin: 0 0 var(--space-2);
    word-break: break-all;
  }

  select {
    width: 100%;
    background: var(--surface);
    color: var(--fg);
    border: none;
    border-radius: var(--radius);
    padding: var(--space-2);
    font-size: 0.85rem;
  }

  .new-item {
    display: flex;
    gap: var(--space-1);
    margin-top: var(--space-1);
  }
  .new-item input {
    flex: 1;
    min-width: 0;
    background: var(--surface);
    border: none;
    border-radius: var(--radius);
    color: var(--fg);
    font-size: 0.8rem;
    padding: var(--space-1) var(--space-2);
  }
  .new-item button {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 0.95rem;
    padding: 0 var(--space-1);
  }
  .new-item button:hover {
    color: var(--accent);
  }

  .empty {
    color: var(--muted);
    font-size: 0.85rem;
  }

  .chip-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .chip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface);
    border-radius: var(--radius);
    padding: var(--space-1) var(--space-2);
  }
  .chip a {
    color: var(--fg);
    text-decoration: none;
    font-size: 0.82rem;
  }
  .chip button {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 0.85rem;
  }
  .chip button:hover {
    color: var(--accent);
  }

  .delete-form {
    margin-top: var(--space-5);
  }
  .delete-btn {
    background: none;
    border: 1px solid var(--muted);
    color: var(--muted);
    border-radius: var(--radius);
    padding: var(--space-2);
    width: 100%;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .delete-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
