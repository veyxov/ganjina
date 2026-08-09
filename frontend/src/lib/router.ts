// Hand-rolled — three route shapes total (home, collection, asset detail) don't
// justify a router dependency. Plain history API + a reactive path.

export type Route =
  | { name: 'home' }
  | { name: 'collection'; id: string }
  | { name: 'asset'; id: string };

function parse(path: string): Route {
  const collectionMatch = path.match(/^\/collections\/([^/]+)\/?$/);
  if (collectionMatch) return { name: 'collection', id: collectionMatch[1] };

  const assetMatch = path.match(/^\/assets\/([^/]+)\/?$/);
  if (assetMatch) return { name: 'asset', id: assetMatch[1] };

  return { name: 'home' };
}

let route = $state<Route>(parse(window.location.pathname));

window.addEventListener('popstate', () => {
  route = parse(window.location.pathname);
});

export function currentRoute() {
  return route;
}

export function navigate(path: string) {
  window.history.pushState({}, '', path);
  route = parse(path);
}
