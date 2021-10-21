const version = 'v1::';

// cache entire app
self.addEventListener('install', event => event.waitUntil(
    caches.open(version + 'fundamentals').then(cache => cache.addAll([
        '/',
        '/global.css',
        '/build/bundle.css',
        '/build/bundle.js',
        '/build/bundle.js.map',
        '/manifest.json',
        '/favicon.png',
        '/service-worker.js',
    ]))
));

// handle fetch events
self.addEventListener('fetch', event => {
    // return if request isn't get
    if (event.request.method !== 'GET') {
        return;
    }
    // respond to request
    event.respondWith(caches.match(event.request).then( async local => {
        // try to send back remote response
        try {
            const remote = await fetch(event.request.clone());
            // update local cache if response was already cached
            if (local) {
                caches.open(version + 'fundamentals').then(cache => cache.put(event.request, remote));
            }
            return remote.clone();
        } catch {}
        // try to send back local response
        if (local) {
            return local;
        }
        // send back default response
        return new Response('<h1>Service Unavailable</h1>', {
            status: 503,
            statusText: 'Service Unavailable',
            headers: new Headers({
                'Content-Type': 'text/html'
            }),
        });
    }));
});