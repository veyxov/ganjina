<script lang="ts">
  import type { Collection } from './types';
  import { api } from './api';
  import { navigate, currentRoute } from './router.svelte';
  import { sidebarCollapsed } from './sidebarState.svelte';

  let { collections, onCollectionsChanged }: { collections: Collection[]; onCollectionsChanged: () => void } =
    $props();

  let newName = $state('');

  async function createCollection(e: Event) {
    e.preventDefault();
    const name = newName.trim();
    if (!name) return;
    const created = await api.createCollection(name);
    newName = '';
    onCollectionsChanged();
    navigate(`/collections/${created.id}`);
  }

  function go(path: string, e: MouseEvent) {
    e.preventDefault();
    navigate(path);
  }

  const route = $derived(currentRoute());
  const collapsed = $derived(sidebarCollapsed());
</script>

<aside class="sidebar" class:collapsed>
  <div class="brand">Ganjina</div>
  <ul class="nav">
    <li>
      <a href="/" class:active={route.name === 'home'} onclick={(e) => go('/', e)}>Photos</a>
    </li>
  </ul>

  <div class="nav-label">Collections</div>
  <ul class="nav">
    {#each collections as c (c.id)}
      <li>
        <a
          href="/collections/{c.id}"
          class:active={route.name === 'collection' && route.id === c.id}
          onclick={(e) => go(`/collections/${c.id}`, e)}
        >
          {c.name}
        </a>
      </li>
    {/each}
  </ul>
  <form class="new-item" onsubmit={createCollection}>
    <input type="text" placeholder="New collection" bind:value={newName} />
    <button type="submit" title="Create">+</button>
  </form>
</aside>

<style>
  .sidebar {
    flex: none;
    width: var(--sidebar-w);
    background: var(--sidebar-bg);
    padding: var(--space-4) var(--space-3);
    overflow: hidden;
    transition: width 0.2s ease, padding 0.2s ease, opacity 0.15s ease;
  }
  .sidebar.collapsed {
    width: 0;
    padding-left: 0;
    padding-right: 0;
    opacity: 0;
    pointer-events: none;
  }

  .brand {
    font-size: 0.95rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    margin-bottom: var(--space-5);
    padding: 0 var(--space-2);
  }

  .nav {
    list-style: none;
    padding: 0;
    margin: 0 0 var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .nav a {
    display: block;
    padding: var(--space-2);
    border-radius: var(--radius);
    color: var(--muted);
    text-decoration: none;
    font-size: 0.88rem;
    transition: background 0.15s ease, color 0.15s ease;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .nav a:hover {
    background: var(--surface-hover);
    color: var(--fg);
  }
  .nav a.active {
    background: var(--surface);
    color: var(--accent);
    font-weight: 600;
  }

  .nav-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    padding: 0 var(--space-2);
    margin-bottom: var(--space-1);
  }

  .new-item {
    display: flex;
    gap: var(--space-1);
    padding: 0 var(--space-2);
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
  .new-item input::placeholder {
    color: var(--muted);
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
</style>
