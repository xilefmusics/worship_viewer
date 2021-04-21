import { List, ListItem, ListItemText } from "@material-ui/core";
import useStyles from "./style";

const TitleList = ({ titles, selectTitle, idx }) => {
  const classes = useStyles();

  return (
    <div className={classes.TitleList}>
      <List>
        {titles.map((title, title_idx) => (
          <ListItem
            selected={idx === title_idx}
            button
            key={title_idx}
            onClick={() => selectTitle(title, title_idx)}
          >
            <ListItemText>{title}</ListItemText>
          </ListItem>
        ))}
      </List>
    </div>
  );
};

export default TitleList;
