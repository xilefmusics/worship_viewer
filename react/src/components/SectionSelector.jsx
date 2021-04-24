import { Grid, Card, CardContent, Typography } from "@material-ui/core";

const SectionSelector = ({ song, selectSection, clearBeamer }) => {
  if (song === null) {
    return null;
  }

  song.sections.forEach((section, idx) => (section.idx = idx));

  return (
    <div
      style={{ padding: "16px", overflow: "auto", height: "100vh" }}
      onDoubleClick={clearBeamer}
    >
      <Grid container spacing={2} justify="flex-start">
        {song.sections
          .filter(
            (section) => section.lines.filter((line) => line.text).length > 0
          )
          .map((section, section_idx) => (
            <Grid item key={section_idx} xs={12} md={6} lg={4}>
              <Card>
                <CardContent
                  style={{ height: "30vh" }}
                  onClick={() => selectSection(section.idx)}
                >
                  <Typography variant="h6" gutterBottom>
                    {section.keyword}
                  </Typography>
                  {section.lines.map((line, line_idx) => {
                    return (
                      <Typography key={line_idx}>
                        {line.text === null ? "" : line.text}
                      </Typography>
                    );
                  })}
                </CardContent>
              </Card>
            </Grid>
          ))}
      </Grid>
    </div>
  );
};

export default SectionSelector;
