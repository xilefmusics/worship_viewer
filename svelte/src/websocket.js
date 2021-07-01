//const url = "ws://localhost:8001";
const url = `ws://${window.location.hostname}:8001`;

const ws = new WebSocket(url);
const wsID = Math.random().toString(36).substr(2, 9);

const sendLoadSetlist = (title) => {
  ws.send(
    JSON.stringify({
      type: "load setlist",
      title: title,
      senderID: wsID,
    })
  );
};

const sendLoadSong = (title, key) => {
  ws.send(
    JSON.stringify({
      type: "load song",
      title: title,
      key: key,
      senderID: wsID,
    })
  );
};

const sendDisplaySection = (title, idx) => {
  ws.send(
    JSON.stringify({
      type: "display section",
      title: title,
      idx: idx,
      senderID: wsID,
    })
  );
};

const sendClearBeamer = () => {
  ws.send(
    JSON.stringify({
      type: "clear beamer",
      senderID: wsID,
    })
  );
};

const sendChangeKey = (key) => {
  ws.send(
    JSON.stringify({
      type: "change key",
      senderID: wsID,
      key: key,
    })
  );
};

export { ws, wsID, sendLoadSetlist, sendLoadSong, sendDisplaySection, sendClearBeamer, sendChangeKey };
