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
          event.waitUntil(
            caches.open(CACHE_NAME).then((cache) => cache.put(req, res.clone()))
          );
        }
        return res;
      })
      .catch(() =>
        caches.match(req).then((cached) =>
          cached ||
          new Response("Offline", {
            status: 503,
            headers: { "Content-Type": "text/plain" },
          })
        )
      )
  );
});
