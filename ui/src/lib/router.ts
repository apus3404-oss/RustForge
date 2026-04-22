import { writable, derived } from 'svelte/store';

export interface Route {
  path: string;
  params: Record<string, string>;
}

function parseHash(): Route {
  const hash = window.location.hash.slice(1) || '/';
  const [path, ...rest] = hash.split('?');

  // Parse path parameters (e.g., /workflows/123 -> { id: '123' })
  const segments = path.split('/').filter(Boolean);
  const params: Record<string, string> = {};

  return { path, params };
}

function createRouter() {
  const { subscribe, set } = writable<Route>(parseHash());

  function updateRoute() {
    set(parseHash());
  }

  // Listen to hash changes
  window.addEventListener('hashchange', updateRoute);

  return {
    subscribe,
    navigate: (path: string) => {
      window.location.hash = path;
    }
  };
}

export const router = createRouter();

// Derived store for current path
export const currentPath = derived(router, $router => $router.path);

// Helper to match routes with parameters
export function matchRoute(pattern: string, path: string): Record<string, string> | null {
  const patternSegments = pattern.split('/').filter(Boolean);
  const pathSegments = path.split('/').filter(Boolean);

  if (patternSegments.length !== pathSegments.length) {
    return null;
  }

  const params: Record<string, string> = {};

  for (let i = 0; i < patternSegments.length; i++) {
    const patternSeg = patternSegments[i];
    const pathSeg = pathSegments[i];

    if (patternSeg.startsWith(':')) {
      // Parameter segment
      params[patternSeg.slice(1)] = pathSeg;
    } else if (patternSeg !== pathSeg) {
      // Literal segment doesn't match
      return null;
    }
  }

  return params;
}

export function navigate(path: string) {
  router.navigate(path);
}
