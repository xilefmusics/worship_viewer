import { Typography, Box } from "@material-ui/core";
import useStyles from "./style";

const SongView = ({ song }) => {
  const classes = useStyles();

  if (!song) {
    return null;
  }

  return (
    <div className={classes.SongView}>
      {song.sections.map((section, section_idx) => (
        <div key={section_idx}>
          <Box fontFamily="Monospace"> </Box>
          <Box fontFamily="Monospace" fontWeight="fontWeightBold">
            {section.keyword}
          </Box>
          {section.lines.map((line, line_idx) => (
            <div key={line_idx}>
              <Box fontFamily="Monospace" fontWeight="fontWeightBold">
                {" "}
                {line.chord}
              </Box>
              <Box fontFamily="Monospace"> {line.text}</Box>
              <Box fontFamily="Monospace">{line.translation}</Box>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};

export default SongView;
