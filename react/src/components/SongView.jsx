import { Box } from "@material-ui/core";

const SongView = ({ song, nextSong, prevSong, toggleSongSelector }) => {
  if (song === null) {
    return null;
  }

  return (
    <div
      style={{
        whiteSpace: "pre",
        height: "100vh",
        overflow: "auto",
        paddingLeft: "1em",
        paddingBottom: "1em",
      }}
      onClick={(event => {
        console.log(event);
        if (event.clientY < window.innerHeight/4) {
          prevSong();
        } else if (event.clientY > window.innerHeight*3/4) {
          nextSong();
        } else if (event.clientX < window.innerWidth / 2) {
          toggleSongSelector();
        }
      })}
    >
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
