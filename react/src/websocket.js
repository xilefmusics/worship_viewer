//const url = "ws://localhost:8001";
const url = `ws://${window.location.hostname}:8001`;

const ws = new WebSocket(url);

const sendLoadSong = (title, key) => {
  ws.send(
    JSON.stringify({
      type: "load song",
      title: title,
      key: key,
    })
  );
};

const sendDisplaySection = (title, idx) => {
  ws.send(
    JSON.stringify({
      type: "display section",
      title: title,
      idx: idx,
    })
  );
};

const sendClearBeamer = () => {
  ws.send(
    JSON.stringify({
      type: "clear beamer",
    })
  );
};

export { ws, sendLoadSong, sendDisplaySection, sendClearBeamer };
