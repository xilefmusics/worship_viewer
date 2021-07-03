const url = "ws://localhost:8001";
//const url = `ws://${window.location.hostname}:8001`;

const ws = new WebSocket(url);
const wsID = Math.random().toString(36).substr(2, 9);
let wsConfig = {
  sendControls: false,
  receiveControls: false,
};

const sendLoadSetlist = (title) => {
  if (!wsConfig.sendControls) {
    return;
  }
  ws.send(
    JSON.stringify({
      type: "load setlist",
      title: title,
      senderID: wsID,
    })
  );
};

const sendLoadSong = (title, key) => {
  if (!wsConfig.sendControls) {
    return;
  }
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
  if (!wsConfig.sendControls) {
    return;
  }
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
  if (!wsConfig.sendControls) {
    return;
  }
  ws.send(
    JSON.stringify({
      type: "clear beamer",
      senderID: wsID,
    })
  );
};

const sendChangeKey = (key) => {
  if (!wsConfig.sendControls) {
    return;
  }
  ws.send(
    JSON.stringify({
      type: "change key",
      senderID: wsID,
      key: key,
    })
  );
};

export { ws, wsID, sendLoadSetlist, sendLoadSong, sendDisplaySection, sendClearBeamer, sendChangeKey, wsConfig };
