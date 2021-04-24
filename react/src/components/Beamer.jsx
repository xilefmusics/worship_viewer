import { Typography } from "@material-ui/core";

const Beamer = ({ section, title }) => {
  if (!section || !title) {
    return null;
  }
  return (
    <div style={{ height: "100vh", padding: "18px" }}>
      <Typography variant="h4">
        {title} ({section.keyword})
      </Typography>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          height: "100%",
        }}
      >
        <div>
          {section.lines.map((line, line_idx) => (
            <Typography key={line_idx} variant="h2" align="center">
              {line.text === null ? "" : line.text}
            </Typography>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Beamer;
