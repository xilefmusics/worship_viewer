const version = 'v1::';

// cache entire app
self.addEventListener('install', event => event.waitUntil(
    caches.open(version + 'fundamentals').then(cache => cache.addAll([
        '/',
        '/static/global.css',
        '/static/bundle.css',
        '/static/bundle.js.map',
        '/static/manifest.json',
        '/static/favicon.png',
        '/static/service-worker.js',
    ]))
));

// handle fetch events
self.addEventListener('fetch', async event => {
    if (event.request.method !== 'GET') {
        return;
    }

    console.log('try getting from remote');
    try {
        const remote = await fetch(event.request);
        caches.open(version + 'fundamentals').then(cache => cache.put(event.request, remote));
        console.log(remote);
        return remote;
    } catch {}

    console.log('try getting from cache');
    try {
        const local = await caches.match(event.request);
        console.log(local);
        return local;
    } catch {}

    console.log('not in remote and not in cache');
    return new Response('<h1>Service Unavailable</h1>', {
        status: 503,
        statusText: 'Service Unavailable',
        headers: new Headers({
          'Content-Type': 'text/html'
        })
      });
});

// remove older versions of service worker
self.addEventListener("activate", async event => {
    console.log("ONACTIVATE: start");
    const keys = cache.keys();
    Promise.all(keys.filter(key => !key.startsWith(version)).map(key => caches.delete(key)));
    console.log("ONACTIVATE: finished");
});