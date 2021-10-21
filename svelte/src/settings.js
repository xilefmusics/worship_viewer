import Dexie from 'dexie';
const db = new Dexie('settings');
db.version(1).stores({
    settings: 'key,value',
})

const set = (key, value) => db.settings.update(key, {value: value}).then((updated) => {
    if (!updated) {
        db.settings.put({key: key, value: value});
    }
});

const get = async key => (await db.settings.get(key)).value;

export default {
    storeApiUrl: value => set('apiUrl', value),
    storeApiPort: value => set('apiPort', value),
    storeCommunicationUrl: value => set('communicationUrl', value),
    storeCommunicationPort: value => set('communicationPort', value),
    storeSendControls: value => set('sendControls', value ? 'true' : 'false'),
    storeReceiveControls: value => set('receiveControls', value ? 'true' : 'false'),
    storeFontScale: value => set('fontScale', value),
    loadApiUrl: async () => ((await get('apiUrl')) || window.location.hostname),
    loadApiPort: async () => ((await get('apiPort')) || 8000),
    loadCommunicationUrl: async () => ((await get('communicationUrl')) || window.location.hostname),
    loadCommunicationPort: async () => ((await get('communicationPort')) || 8001),
    loadSendControls: async () => (await get('sendControls')),
    loadReceiveControls: async () => (await get('receiveControls')),
    loadFontScale: async () => ((await get('fontScale')) || 0.8),
};