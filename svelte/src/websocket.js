let ws = null;

const wsID = Math.random().toString(36).substr(2, 9);
const wsProtocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
let wsConfig = {
  sendControls: true,
  receiveControls: true,
};

const sendLoadSetlist = (title) => {
  if (!wsConfig.sendControls || !ws || ws.readyState != WebSocket.OPEN) {
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
  if (!wsConfig.sendControls || !ws || ws.readyState != WebSocket.OPEN) {
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
  if (!wsConfig.sendControls || !ws || ws.readyState != WebSocket.OPEN) {
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
  if (!wsConfig.sendControls || !ws || ws.readyState != WebSocket.OPEN) {
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
  if (!wsConfig.sendControls || !ws || ws.readyState != WebSocket.OPEN) {
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

const wsChangeUrl = (new_url, new_port) => ws = new WebSocket(`${wsProtocol}//${new_url}:${new_port}`);

export { ws, wsID, sendLoadSetlist, sendLoadSong, sendDisplaySection, sendClearBeamer, sendChangeKey, wsConfig, wsChangeUrl };
