import { List, ListItem, ListItemText } from "@material-ui/core";

const TitleList = ({ titles, selectTitle, idx }) => {
  return (
    <div
      style={{
        height: "100%",
        overflow: "auto",
      }}
    >
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
