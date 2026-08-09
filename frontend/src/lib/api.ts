import type { Asset, Collection, Owner, PhotoMetadata, AdjacentAssets, StatusSummary } from './types';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api${path}`, {
    headers: init?.body && !(init.body instanceof FormData) ? { 'Content-Type': 'application/json' } : undefined,
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error ?? `request failed: ${res.status}`);
  }
  if (res.status === 204 || res.headers.get('content-length') === '0') {
    return undefined as T;
  }
  return res.json();
}

export const api = {
  listAssets: () => request<Asset[]>('/assets'),
  getAsset: (id: string) => request<Asset>(`/assets/${id}`),
  getMetadata: (id: string) => request<PhotoMetadata | null>(`/assets/${id}/metadata`),
  getAdjacent: (id: string) => request<AdjacentAssets>(`/assets/${id}/adjacent`),
  getStatus: () => request<StatusSummary>('/status'),
  deleteAsset: (id: string) => request<void>(`/assets/${id}`, { method: 'DELETE' }),
  setOwner: (id: string, ownerId: string | null) =>
    request<void>(`/assets/${id}/owner`, { method: 'PUT', body: JSON.stringify({ owner_id: ownerId }) }),
  upload: (files: FileList): Promise<Asset[]> => {
    const form = new FormData();
    for (const file of files) form.append('file', file);
    return request<Asset[]>('/assets', { method: 'POST', body: form });
  },

  listCollections: () => request<Collection[]>('/collections'),
  createCollection: (name: string) =>
    request<Collection>('/collections', { method: 'POST', body: JSON.stringify({ name }) }),
  getCollection: (id: string) => request<Collection>(`/collections/${id}`),
  listCollectionAssets: (id: string) => request<Asset[]>(`/collections/${id}/assets`),
  addAssetToCollection: (collectionId: string, assetId: string) =>
    request<void>(`/collections/${collectionId}/assets`, {
      method: 'POST',
      body: JSON.stringify({ asset_id: assetId }),
    }),
  removeAssetFromCollection: (collectionId: string, assetId: string) =>
    request<void>(`/collections/${collectionId}/assets/${assetId}`, { method: 'DELETE' }),
  collectionsForAsset: (assetId: string) => request<Collection[]>(`/assets/${assetId}/collections`),

  listOwners: () => request<Owner[]>('/owners'),
  createOwner: (name: string) => request<Owner>('/owners', { method: 'POST', body: JSON.stringify({ name }) }),
};

export function humanSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${bytes} B` : `${size.toFixed(1)} ${units[unit]}`;
}

export function monthLabel(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
}
