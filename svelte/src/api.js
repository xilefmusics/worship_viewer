import offline from './offline/api';
let url = '';

const fetchTitles = async () => {
  if (url == 'offline') {
    return offline.getTitles();
  }
  try {
    const response = await fetch(`${url}/song_titles`);
    const json = await response.json();
    return json;
  } catch(e) {
    return [];
  }
};

const fetchSong = async (title, key) => {
  if (url == 'offline') {
    return offline.getSong(title, key);
  }
  try {
    const response = await fetch(`${url}/song/${title}/${key}`);
    const json = await response.json();
    return json;
  } catch (e) {
    return null;
  }
};

const fetchSetlists = async () => {
  if (url == 'offline') {
    return offline.getSetlists();
  }
  try {
    const response = await fetch(`${url}/setlist_titles`);
    const json = await response.json();
    return json;
  } catch (e) {
    return [];
  }
}

const fetchFirstSetlist = async () => {
  if (url == 'offline') {
    return offline.getFirstSetlist();
  }
  const response = await fetch(`${url}/setlist_get_first`);
  const json = await response.json();
  return json;
}

const fetchSetlist = async (title) => {
  if (!title) {
    return getFirstSetlist();
  }
  if (url == 'offline') {
    return offline.getSetlist(title);
  }
  const response = await fetch(`${url}/setlist/${title}`);
  const json = await response.json();
  return json;
}

const apiChangeUrl = (new_url, new_port) =>{
  if (new_url == 'offline') {
    url = 'offline';
  } else {
    url = `http://${new_url}:${new_port}`;
  }
};

const makeOffline = async () => {
  offline.clearSongs();
  const titles = await fetchTitles();
  await Promise.all(
    titles.map(async (title) => offline.addSong(await fetchSong(title, 'Self')))
  );
  offline.clearSetlists();
  const setlists = await fetchSetlists();
  await Promise.all(
    setlists.map(async (title) => offline.addSetlist(await fetchSetlist(title)))
  );
};


export { fetchTitles, fetchSong, fetchSetlists, fetchSetlist, apiChangeUrl, makeOffline };
