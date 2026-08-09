<script lang="ts">
  import type { Asset, Collection } from './lib/types';
  import { api } from './lib/api';
  import { currentRoute } from './lib/router';
  import { toggleSidebar } from './lib/sidebar';
  import Sidebar from './lib/Sidebar.svelte';
  import Gallery from './lib/Gallery.svelte';
  import CollectionView from './lib/views/CollectionView.svelte';
  import AssetDetail from './lib/views/AssetDetail.svelte';

  let collections = $state<Collection[]>([]);
  let assets = $state<Asset[]>([]);
  let failedCount = $state(0);

  async function loadCollections() {
    collections = await api.listCollections();
  }

  async function loadHome() {
    const [a, s] = await Promise.all([api.listAssets(), api.getStatus()]);
    assets = a;
    failedCount = s.failed_count;
  }

  loadCollections();
  loadHome();

  const route = $derived(currentRoute());
</script>

<div class="shell">
  <Sidebar {collections} onCollectionsChanged={loadCollections} />
  <main class="content">
    <div class="topbar">
      <button class="sidebar-toggle" onclick={toggleSidebar} title="Toggle sidebar">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
          <path d="M4 6h16M4 12h16M4 18h16" stroke-linecap="round" />
        </svg>
      </button>
    </div>
    <div class="page">
      {#if route.name === 'home'}
        <Gallery title="Photos" {assets} {failedCount} showUpload onChanged={loadHome} />
      {:else if route.name === 'collection'}
        {#key route.id}
          <CollectionView id={route.id} />
        {/key}
      {:else if route.name === 'asset'}
        {#key route.id}
          <AssetDetail id={route.id} />
        {/key}
      {/if}
    </div>
  </main>
</div>

<style>
  .shell {
    display: flex;
    min-height: 100vh;
  }
  .content {
    flex: 1;
    min-width: 0;
    padding: var(--space-4) var(--space-4) var(--space-5);
  }
  .topbar {
    margin-bottom: var(--space-3);
  }
  .sidebar-toggle {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    border-radius: var(--radius);
  }
  .sidebar-toggle:hover {
    background: var(--surface);
    color: var(--fg);
  }
  .sidebar-toggle svg {
    width: 16px;
    height: 16px;
  }
  .page {
    max-width: 960px;
    margin: 0 auto;
  }
</style>
