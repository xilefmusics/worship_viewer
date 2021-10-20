import Dexie from 'dexie';
const db = new Dexie('offline');
db.version(1).stores({
    songs: 'title,data',
    setlists: 'title, data',
})

const addSong = async (song) => db.songs.put({title: song.title, data: JSON.stringify(song)});
const getTitles = async () => await db.songs.orderBy('title').keys();
const getSong = async (title, key) => await JSON.parse((await db.songs.get(title)).data);
const clearSongs = async () => await db.songs.clear();

const addSetlist = async (setlist) => db.setlists.put({title: setlist.title, data: JSON.stringify(setlist)});
const getSetlists = async () => await db.setlists.orderBy('title').keys();
const getSetlist = async (title) => await JSON.parse((await db.setlists.get(title)).data);
const clearSetlists = async () => await db.setlists.clear();

export default {getTitles, getSong, getSetlists, getSetlist, addSong, addSetlist, clearSongs, clearSetlists};