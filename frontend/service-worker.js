const CACHE_NAME = "offline-cache-v1";
const DATA_CACHE_NAME = "offline-data-cache-v1";

function shouldCacheApiRequest(pathname) {
  const listPattern = /^\/api\/v1\/(setlists|collections)$/;
  const idPattern = /^\/api\/v1\/(setlists|collections)\/[^\/]+\/(player|songs)$/;
  return listPattern.test(pathname) || idPattern.test(pathname);
}

self.addEventListener("fetch", (event) => {
  const req = event.request;

  if (req.method !== "GET") return;

  const url = new URL(req.url);

  if (shouldCacheApiRequest(url.pathname)) {
    event.respondWith(
      fetch(req)
        .then((res) => {
          if (res.ok) {
            const resForCache = res.clone();
            caches.open(DATA_CACHE_NAME).then((cache) => {
              cache.put(req, resForCache).catch(() => {});
            }).catch(() => {});
          }
          return res;
        })
        .catch(() => {
          return caches.match(req).then((cached) => {
            if (cached) {
              return cached;
            } else {
              return new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
            }
          });
        })
    );
    return;
  }

  if (url.pathname.startsWith("/api/")) {
    return;
  }

  event.respondWith(
    fetch(req)
      .then((res) => {
        if (res.ok) {
          const resForCache = res.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(req, resForCache).catch(() => {});
          }).catch(() => {});
        }
        return res;
      })
      .catch(() => {
        return caches.match(req).then((cached) => {
          if (cached) {
            return cached;
          } else {
            return new Response("Offline", { status: 503, headers: { "Content-Type": "text/plain" } });
          }
        });
      })
  );
});
