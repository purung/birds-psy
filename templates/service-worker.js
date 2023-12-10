// Service Worker Script

// 1. Version Control
const CURRENT_CACHE = "<%= version %>"; // Change this version number when updating the service worker
const PUSH_SERVER_URL = '/api/push-me';
const BASE64_PUBKEY = "BA4NZv33DCXlOm1TUWLEiMM5n9GVar0BC7xyZWaCnbIS0PXsRNr-NsgojCzyITywztw5N_YSnFLoHJ7rwzLcLD0"

// Assets to cache (can be an array of file URLs)
const assetsToCache = [
    '/pkg/birds-psy.css',
    '/pkg/birds-psy.js',
    '/pkg/birds-psy.wasm',
    // Add more assets as required
];

// 2. Install Event - Caches assets and notifies about the update
self.addEventListener('install', event => {
    console.log('Service Worker: Installing...');

    // Cache assets
    if (CURRENT_CACHE === "DEV") {
        self.skipWaiting();
        return
    };
    event.waitUntil(
        caches.open(CURRENT_CACHE)
            .then(cache => {
                console.log('Service Worker: Caching files');
                cache.addAll(assetsToCache);
            })
            .then(() => {
                // 3. Automatic Activation - Uncomment next line to enable
                self.skipWaiting();
            })
            .catch(err => console.error('Service Worker: Caching failed with ' + err))
    );
});

// 4. Activate Event - Clear old caches and control activation
self.addEventListener('activate', event => {
    console.log('Service Worker: Activated');

    // Remove old caches
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames.map(cache => {
                    if (cache !== CURRENT_CACHE) {
                        console.log('Service Worker: Clearing Old Cache', cache);
                        return caches.delete(cache);
                    }
                })
            );
        })
    );

    // 4. Controlled Activation - Ensures service worker activation is controlled
    event.waitUntil(self.clients.claim());
});

// 5. Fetch Event - Serve cached content when offline
self.addEventListener('fetch', event => {
    console.log('Service Worker: Fetching');
    event.respondWith(
        caches.match(event.request)
            .then(response => {
                // Return cache hit or fetch from network
                return response || fetch(event.request);
            })
    );
});

// Additional event listeners and logic can be added as needed

// Service Worker for enabling push notifications

// URL for your server that handles subscription data


self.addEventListener('push', event => {
  console.log('Push message received.');
  
  // Customize this with the details of the push message
  const title = 'Ny förfrågan';
  const options = {
    body: event.data.text(),
    icon: '/icons/raccoon-512.png',
    badge: '/icons/racoon-72.png'
  };

  event.waitUntil(self.registration.showNotification(title, options));
});

self.addEventListener('notificationclick', event => {
  console.log('Notification click received.');

  event.notification.close();

  const urlToOpen = new URL("/", self.location.origin).href;
    console.log(urlToOpen);

  // Handle notification click, for example, by opening a URL
  event.waitUntil(
    clients.openWindow(urlToOpen)
  );
});

self.addEventListener('pushsubscriptionchange', event => {
  console.log('Subscription change event');
  event.waitUntil(
    self.registration.pushManager.subscribe({ userVisibleOnly: true })
      .then(newSubscription => {
        // Send new subscription details to server
        return sendSubscriptionToServer(newSubscription)
      })
  );
});

self.addEventListener('message', function (messageEvent) {
    if (messageEvent.data === "skipWaiting") {
        console.log("Service-worker received skipWaiting event");
        self.skipWaiting();
    }
});

