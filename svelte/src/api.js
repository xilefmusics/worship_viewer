let url = '';

const fetchTitles = async () => {
  try {
    const response = await fetch(`${url}/song_titles`);
    const json = await response.json();
    return json;
  } catch(e) {
    return [];
  }
};

const fetchSong = async (title, key) => {
  try {
    const response = await fetch(`${url}/song/${title}/${key}`);
    const json = await response.json();
    return json;
  } catch (e) {
    return null;
  }
};

const fetchSetlists = async () => {
  try {
    const response = await fetch(`${url}/setlist_titles`);
    const json = await response.json();
    return json;
  } catch (e) {
    return [];
  }
}

const fetchFirstSetlist = async () => {
    const response = await fetch(`${url}/setlist_get_first`);
    const json = await response.json();
    return json;
}

const fetchSetlist = async (title) => {
  if (!title) {
    return fetchFirstSetlist();
  }
  const response = await fetch(`${url}/setlist/${title}`);
  const json = await response.json();
  return json;
}

const apiChangeUrl = (new_url, new_port) => url = `http://${new_url}:${new_port}`;


export { fetchTitles, fetchSong, fetchSetlists, fetchSetlist, apiChangeUrl };
