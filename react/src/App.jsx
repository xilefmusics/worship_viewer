import React, { useState, useEffect } from "react";
import { Grid } from "@material-ui/core";
import { fetchTitles, fetchSong } from "./api";
import useStyles from "./style";
import TitleList from "./TitleList";
import SongView from "./SongView";

const App = () => {
  const classes = useStyles();
  const [titles, setTitles] = useState([]);
  const [song, setSong] = useState({
    idx: null,
    content: null,
  });
  const [key, setKey] = useState(null);

  document.onkeydown = (e) => {
    if (e.key == " " || e.key == "j") {
      const idx = song.idx + 1;
      if (idx >= titles.length) {
        return;
      }
      const title = titles[idx];
      fetchSong(title, key).then((song) => setSong({ content: song, idx }));
    } else if (e.key == "k") {
      const idx = song.idx - 1;
      if (idx < 0) {
        return;
      }
      const title = titles[idx];
      fetchSong(title, key).then((song) => setSong({ content: song, idx }));
    } else if (
      e.key === "A" ||
      e.key === "B" ||
      e.key === "C" ||
      e.key === "D" ||
      e.key === "E" ||
      e.key === "F" ||
      e.key === "G"
    ) {
      setKey(e.key);
    } else if (e.key === "b" || e.key === "#") {
      if (key != null) {
        setKey(`${key.substring(0, 1)}${e.key}`);
      }
    }
  };

  useEffect(() => {
    const inner = async () => {
      const titles = await fetchTitles();
      const idx = song.idx ? song.idx : 0;
      const content = await fetchSong(titles[idx], key);
      setTitles(titles);
      setSong({ content, idx });
    };
    inner();
  }, [key]);

  return (
    <div className={classes.App}>
      <Grid container spacing={0} alignItems="stretch">
        <Grid item>
          <TitleList
            titles={titles}
            idx={song.idx}
            selectTitle={(title, idx) =>
              fetchSong(title, key).then((song) =>
                setSong({ content: song, idx })
              )
            }
          />
        </Grid>
        <Grid item className={classes.grid_item_SongView}>
          <SongView song={song.content} />
        </Grid>
      </Grid>
    </div>
  );
};

export default App;
