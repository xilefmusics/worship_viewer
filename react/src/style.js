import React from "react";
import { makeStyles } from "@material-ui/core/styles";

const useStyles = makeStyles((theme) => ({
  App: {
    height: "100vh",
  },
  SongView: {
    whiteSpace: "pre",
    height: "100vh",
    overflow: "auto",
    paddingLeft: "1em",
    paddingBottom: "1em",
  },
  TitleList: {
    height: "100vh",
    overflow: "auto",
  },
  grid_item_SongView: {
    flexGrow: 1,
  },
}));

export default useStyles;
