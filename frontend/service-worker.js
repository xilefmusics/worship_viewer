const CACHE_NAME = 'worship-viewer-static-v1';
const CORE_ASSETS = ['/', '/index.html', '/manifest.json', '/favicon.png'];

self.addEventListener('install', (event) => {
    self.skipWaiting();
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(CORE_ASSETS);
        }),
    );
});

self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((keys) => {
            return Promise.all(
                keys
                    .filter((key) => key !== CACHE_NAME)
                    .map((key) => caches.delete(key)),
            );
        }),
    );
    self.clients.claim();
});

const shouldHandleFetch = (request) => {
    if (request.method !== 'GET') {
        return false;
    }

    const url = new URL(request.url);

    if (url.origin !== self.location.origin) {
        return false;
    }

    if (url.pathname.startsWith('/api/')) {
        return false;
    }

    return true;
};

self.addEventListener('fetch', (event) => {
    if (!shouldHandleFetch(event.request)) {
        return;
    }

    const { request } = event;

    if (request.mode === 'navigate') {
        event.respondWith(
            fetch(request)
                .then((response) => {
                    const copy = response.clone();
                    caches
                        .open(CACHE_NAME)
                        .then((cache) => cache.put('/index.html', copy))
                        .catch(() => {});
                    return response;
                })
                .catch(() => caches.match('/index.html')),
        );
        return;
    }

    event.respondWith(
        caches.match(request).then((cachedResponse) => {
            if (cachedResponse) {
                return cachedResponse;
            }

            return fetch(request)
                .then((response) => {
                    if (
                        !response ||
                        response.status !== 200 ||
                        response.type === 'opaque'
                    ) {
                        return response;
                    }

                    const responseToCache = response.clone();
                    caches
                        .open(CACHE_NAME)
                        .then((cache) => cache.put(request, responseToCache))
                        .catch(() => {});
                    return response;
                })
                .catch(() => caches.match('/index.html'));
        }),
    );
});

