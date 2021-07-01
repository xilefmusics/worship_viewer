//const url = "http://localhost:8000";
const url = window.location.origin;

const fetchTitles = async () => {
  const response = await fetch(`${url}/song_titles`);
  const json = await response.json();
  return json;
};

const fetchSong = async (title, key) => {
  const response = await fetch(`${url}/song/${title}/${key}`);
  const json = await response.json();
  return json;
};

const fetchSetlists = async () => {
  const response = await fetch(`${url}/setlist_titles`);
  const json = await response.json();
  return json;
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

export { fetchTitles, fetchSong, fetchSetlists, fetchSetlist };
