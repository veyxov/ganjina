<script lang="ts">
  import type { Asset, Collection } from '../types';
  import { api } from '../api';
  import Gallery from '../Gallery.svelte';

  let { id }: { id: string } = $props();

  let collection = $state<Collection | null>(null);
  let assets = $state<Asset[]>([]);

  async function load() {
    const [c, a] = await Promise.all([api.getCollection(id), api.listCollectionAssets(id)]);
    collection = c;
    assets = a;
  }

  $effect(() => {
    id;
    load();
  });
</script>

{#if collection}
  <Gallery title={collection.name} {assets} onChanged={load} />
{/if}
