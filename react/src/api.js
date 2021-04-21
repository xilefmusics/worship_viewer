//const url = "http://localhost:8000";
const url = window.location.origin;

const fetchTitles = async () => {
  const response = await fetch(`${url}/titles`);
  const json = await response.json();
  return json;
};

const fetchSong = async (title, key) => {
  const response = await fetch(`${url}/song/${title}/${key}`);
  const json = await response.json();
  return json;
};

export { fetchTitles, fetchSong };