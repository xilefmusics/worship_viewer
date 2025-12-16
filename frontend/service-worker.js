const CACHE_NAME = "offline-cache-v1";

self.addEventListener("fetch", (event) => {
  const req = event.request;

  if (req.method !== "GET") return;

  const url = new URL(req.url);

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
