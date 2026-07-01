// S.M.O.B.B. service worker.
//
// Assets that trunk fingerprints (wasm/js/css) have build-unique URLs, so a new
// deploy is a new URL — those are safe to cache forever (cache-first). The
// entry files that keep a STABLE URL across deploys (index.html, controls.js,
// manifest.json) must be revalidated, or a stale index.html would keep pointing
// at last build's wasm hash and the site would never update. Those go
// network-first with a cached fallback so the game still loads offline.

const CACHE = "smobb-v2";

// Stable-named files that change between deploys and must not be served stale.
function isStableEntry(url) {
  const p = url.pathname;
  return (
    p.endsWith("/") ||
    p.endsWith("/index.html") ||
    p.endsWith("/controls.js") ||
    p.endsWith("/manifest.json")
  );
}

self.addEventListener("install", () => {
  self.skipWaiting();
});

self.addEventListener("activate", (e) => {
  e.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
      )
      .then(() => self.clients.claim())
  );
});

function cacheAndReturn(req, res) {
  if (res && res.status === 200) {
    const copy = res.clone();
    caches.open(CACHE).then((c) => c.put(req, copy));
  }
  return res;
}

self.addEventListener("fetch", (e) => {
  const req = e.request;
  const url = new URL(req.url);
  if (req.method !== "GET" || url.origin !== self.location.origin) {
    return;
  }

  // Navigations + stable entry files: network-first so a new deploy wins,
  // falling back to cache when offline.
  if (req.mode === "navigate" || isStableEntry(url)) {
    e.respondWith(
      fetch(req)
        .then((res) => cacheAndReturn(req, res))
        .catch(() => caches.match(req))
    );
    return;
  }

  // Fingerprinted immutable assets: cache-first.
  e.respondWith(
    caches.match(req).then((cached) => {
      if (cached) return cached;
      return fetch(req)
        .then((res) => cacheAndReturn(req, res))
        .catch(() => cached);
    })
  );
});
