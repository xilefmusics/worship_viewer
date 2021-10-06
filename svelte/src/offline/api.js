let tmp_titles = [];
const tmp_song_map = new Map();
let tmp_setlists = [];
const tmp_setlist_map = new Map();

const getTitles = () => {
    return tmp_titles;
};

const getSong = (title, key) => {
    return tmp_song_map.get(title);
};

const getSetlists = () => {
    return tmp_setlists;
};

const getFirstSetlist = () => {
    return tmp_setlist_map.get(tmp_setlists[0]);
};

const getSetlist = (title) => {
    return tmp_setlist_map.get(title);
};

const addSong = (song) => {
    tmp_titles.push(song.title);
    tmp_song_map.set(song.title, song);
};

const addSetlist = (setlist) => {
    tmp_setlists.push(setlist.title);
    tmp_setlist_map.set(setlist.title, setlist);
};

const clearSongs = () => {
    tmp_titles = [];
    tmp_song_map.clear();
}

const clearSetlists = () => {
    tmp_setlists = [];
    tmp_setlist_map.clear();
}

export default {getTitles, getSong, getSetlists, getFirstSetlist, getSetlist, addSong, addSetlist, clearSongs, clearSetlists};