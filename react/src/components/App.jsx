import React from "react";

import { Grid, CssBaseline } from "@material-ui/core";
import { MuiThemeProvider, createMuiTheme } from "@material-ui/core/styles";

import { fetchTitles, fetchSong } from "../api";
import { ws, sendLoadSong, sendDisplaySection } from "../websocket";

import TitleList from "./TitleList";
import SongView from "./SongView";
import SectionSelector from "./SectionSelector";

const appTheme = createMuiTheme({
  palette: {
    type: "dark",
  },
});

class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      title_idx: 0,
      titles: [],
      song: null,
      key: null,
      display: "SectionSelector",
      //display: "SongView",
    };

    ws.addEventListener("message", (event) => {
      const msg = JSON.parse(event.data);
      if (msg.type === "load song") {
        this.loadSong(msg.title, null, msg.key);
      }
    });

    document.onkeydown = (e) => {
      if (e.key === " " || e.key === "j") {
        this.nextSong();
      } else if (e.key === "k") {
        this.prevSong();
      } else if (
        e.key === "A" ||
        e.key === "B" ||
        e.key === "C" ||
        e.key === "D" ||
        e.key === "E" ||
        e.key === "F" ||
        e.key === "G" ||
        e.key === "b" ||
        e.key === "#" ||
        e.key === "r"
      ) {
        const key = e.key === "r" ? null : e.key;
        this.setKey(key);
      } else if (e.key === "1") {
        this.setDisplay("SongView");
      } else if (e.key === "2") {
        this.setDisplay("SectionSelector");
      }
    };
  }

  setDisplay(display) {
    this.setState(() => ({
      display: display,
    }));
  }

  setKey(key) {
    if (key === "b" || key === "#") {
      if (this.state.key === null) {
        return;
      }
      key = `${this.state.key.substring(0, 1)}${key}`;
    }
    sendLoadSong(this.state.song.title, key);
  }

  async componentDidMount() {
    const titles = await fetchTitles();
    const song = await fetchSong(titles[0]);
    this.setState(() => ({
      title_idx: 0,
      titles: titles,
      song: song,
    }));
  }

  nextSong() {
    if (this.state.title_idx >= this.state.titles.length - 1) {
      return;
    }
    sendLoadSong(this.state.titles[this.state.title_idx + 1]);
  }

  prevSong() {
    if (this.state.title_idx < 1) {
      return;
    }

    sendLoadSong(this.state.titles[this.state.title_idx - 1]);
  }

  async loadSong(title, idx, key) {
    idx = idx != null ? idx : this.state.titles.indexOf(title);
    title = title != null ? title : this.state.titles[idx];
    const song = await fetchSong(title, key);
    this.setState(() => ({
      title_idx: idx,
      song: song,
      key: key,
    }));
  }

  async loadTitles() {
    const titles = await fetchTitles();
    this.setState(() => ({
      titles: titles,
    }));
  }

  render() {
    return (
      <MuiThemeProvider theme={appTheme}>
        <CssBaseline />
        <div>
          <Grid container>
            <Grid item style={{ width: "20vw", height: "100vh" }}>
              <TitleList
                titles={this.state.titles}
                idx={this.state.title_idx}
                selectTitle={(title) => sendLoadSong(title, null)}
              />
            </Grid>
            <Grid item style={{ width: "80vw", height: "100vh" }}>
              {this.state.display === "SongView" && (
                <SongView song={this.state.song} />
              )}
              {this.state.display === "SectionSelector" && (
                <SectionSelector
                  song={this.state.song}
                  selectSection={(idx) =>
                    sendDisplaySection(this.state.song.title, idx)
                  }
                />
              )}
            </Grid>
          </Grid>
        </div>
      </MuiThemeProvider>
    );
  }
}

export default App;
