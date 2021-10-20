<script>
  import { fetchSong, apiChangeUrl, makeOffline } from '../api';
  import { ws, wsID, sendLoadSetlist, sendLoadSong, sendDisplaySection, sendClearBeamer, sendChangeKey, wsConfig, wsChangeUrl } from '../websocket';

  import LeftSidebar from './LeftSidebar.svelte'
  import Center from './Center.svelte'
  import RightSidebar from './RightSidebar.svelte'

  const getIsMobile = () => window.innerWidth < window.innerHeight;

  const version = '0.2.1';

  let beamerViewComponent;
  let leftSidebarComponent;
  let centerComponent;
  let rightSidebarComponent;

  let isMobile = getIsMobile();
  let leftVisible = true;
  const toggleLeft = () => leftVisible = !leftVisible;
  const showLeft = () => leftVisible = true;
  const hideLeft = () => leftVisible = false;
  let rightVisible = false;
  const toggleRight = () => rightVisible = !rightVisible;
  const showRight = () => rightVisible = true;
  const hideRight = () => rightVisible = false;
  let mode = 'musican'; // musican, singer, beamer-control, beamer

  let apiUrl = window.location.hostname;
  let apiPort = '8000';
  let communicationUrl = window.location.hostname;
  let communicationPort = '8001'
  apiChangeUrl(apiUrl, apiPort);
  wsChangeUrl(communicationUrl, communicationPort);

  let currentSong;
  let currentCapo = 0;
  let currentKey = 'Self';
  let fontScale = 0.8;

  const onClickCenter = (event) => {
    if (event.y > window.innerHeight*3/4) {
      leftSidebarComponent.nextSong();
    } else if (event.y < window.innerHeight/4) {
      leftSidebarComponent.prevSong();
    } else if (event.x < window.innerWidth/2) {
      toggleLeft();
    } else if (event.x > window.innerWidth/2) {
      toggleRight();
    }
  };
  const onSongSelect = async (item, isRemote) => {
    if (!isRemote) {
      sendLoadSong(item.title, item.key);
      sendClearBeamer();
    }
    if (isMobile) {
      hideLeft();
    }
    if (!item.key || item.key === 'Self') {
      item.key = currentKey;
    }
    centerComponent.clearBeamer();
    currentSong = await fetchSong(item.title, manipulateKey(item.key, -currentCapo));
  };
  const onSetlistSelect = async (title, isRemote) => {
    if (!isRemote) {
      sendLoadSetlist(title);
    }
  };
  const onModeChange = (m) => {
    mode = m;
    if (isMobile) {
      hideRight();
    }
    if (mode === 'beamer') {
      hideLeft();
      hideRight();
    }
  };
  const onSectionSelect = (idx, isRemote) => {
    if (idx === null) {
      if (!isRemote) {
        sendClearBeamer()
      }
      centerComponent.clearBeamer();
      return;
    }
    if (!isRemote) {
      sendDisplaySection(currentSong.title, idx)
    }
    centerComponent.displayBeamer(idx);
  };
  const keys = ['A', 'Bb', 'B', 'C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab'];
  const manipulateKey = (key, offset) => {
    if (offset === 0) {
      return key;
    }
    if (key === 'Self') {
      return `${key}:${(offset + 12) % 12}`;
    }
    if (offset < 0)  {
      offset += 12;
    }
    return keys[(keys.indexOf(key)+offset) % 12];
  }
  const onKeyChange = (key, isRemote) => {
    if (currentKey == 'Self') {
      currentKey = 'Ab';
    }
    if ( key == '+1' ) {
      key = manipulateKey(currentKey, 1);
    } else if (key == '-1') {
      key = manipulateKey(currentKey, -1);
    }
    currentKey = key;
    if (!isRemote) {
      sendChangeKey(currentKey);
    }
    onSongSelect({title: currentSong.title, key: currentKey}, true);
  }
  const onCapoChange = (capo) => {
    if (currentSong.key.indexOf(':') > -1 ) {
      currentSong.key = 'Self';
    } else {
      currentSong.key = manipulateKey(currentSong.key, currentCapo);
    }
    if (capo === 1) {
      currentCapo = (currentCapo + 1) % 12;
    } else if (capo === -1) {
      currentCapo = (currentCapo + 11) % 12;
    } else if (capo === 0) {
      currentCapo = 0;
    }
    onSongSelect({title: currentSong.title, key: currentSong.key}, true);
  }
  const onFontScaleChange = (update) => {
    if (update === 'reset') {
      fontScale = 0.8;
    } else if (update === 'increment') {
      fontScale = Math.round((fontScale + 0.05 + Number.EPSILON) * 100) / 100;
    } else if (update === 'decrement') {
      fontScale = Math.round((fontScale - 0.05 + Number.EPSILON) * 100) / 100;
    }
  };

  const onApiChange = (url, port) => {
    if (url) {
      apiUrl = url;
    }
    if (port) {
      apiPort = port;
    }
    apiChangeUrl(apiUrl, apiPort);
    leftSidebarComponent.reload();
  }

  const onCommunicationChange = (url, port) => {
    if (url) {
      communicationUrl = url;
    }
    if (port) {
      communicationPort = port;
    }
    wsChangeUrl(communicationUrl, communicationPort);
  }

  const onMakeCurrentApiOffline = async () => {
    if (apiUrl == 'offline') {
      return;
    }
    await makeOffline();
    onApiChange('offline', apiPort);
  }

  ws.addEventListener("message", (event) => {
    if (!wsConfig.receiveControls) {
      return;
    }
    const msg = JSON.parse(event.data);
    if (msg.senderID === wsID) {
      return;
    }
    if (msg.type === "load setlist") {
      onSetlistSelect(msg.title, true);
    } else if (msg.type === "load song") {
      onSongSelect({title: msg.title, key: msg.key}, true);
      leftSidebarComponent.selectTitle(msg.title);
    } else if (msg.type === "display section") {
      onSectionSelect(msg.idx, true);
    } else if (msg.type === "clear beamer") {
      centerComponent.clearBeamer();
    } else if (msg.type === "change key") {
      onKeyChange(msg.key, true);
    }
  });

  document.onkeydown = (e) => {
    if (e.key === " " || e.key === "j" || e.keyCode === 40) {
      leftSidebarComponent.nextSong();
    } else if (e.key === "k" || e.keyCode === 38) {
      leftSidebarComponent.prevSong();
    } else if (
      e.key === "A" ||
      e.key === "B" ||
      e.key === "C" ||
      e.key === "D" ||
      e.key === "E" ||
      e.key === "F" ||
      e.key === "G" ||
      e.key === "b" ||
      e.key === "#" ||
      e.key === "r"
    ) {
      let key = e.key === "r" ? "Self" : e.key;
      if (key === "b" || key === "#") {
        if (currentKey === 'Self') {
          return;
        }
        key = `${currentKey.substring(0, 1)}${key}`;
      }
      onKeyChange(key);
    } else if (e.key === "1") {
      onModeChange('musican');
    } else if (e.key === "2") {
      onModeChange('singer');
    } else if (e.key === "3") {
      onModeChange('beamer-control');
    } else if (e.key === "4") {
      onModeChange('beamer');
    } else if (e.keyCode === 37) {
      toggleLeft();
    } else if (e.keyCode === 39) {
      toggleRight();
    } else if (e.key === '+') {
      onFontScaleChange('increment');
    } else if (e.key === '-') {
      onFontScaleChange('decrement');
    } else if (e.key === '=') {
      onFontScaleChange('reset');
    } else if (e.key === 'c') {
      onCapoChange(1);
    } else if (e.key === 'x') {
      onCapoChange(-1);
    } else if (e.key === 'v') {
      onCapoChange(0);
    }
  };

  window.onresize = () => isMobile = getIsMobile();
</script>

<style>
  #app {
    position: fixed;
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: row;
    flex-wrap: nowrap;
    justify-content: flex-start;
    align-items: stretch;
  }
</style>

<main>
  <div id='app'>
    <LeftSidebar
      onSongSelect={onSongSelect}
      onSetlistSelect={onSetlistSelect}
      visible={leftVisible}
      bind:this={leftSidebarComponent}
      />
    <Center
      onClickCenter={onClickCenter}
      onSectionSelect={onSectionSelect}
      mode={mode}
      currentSong={currentSong}
      fontScale={fontScale}
      visible={!isMobile || (!leftVisible && !rightVisible)}
      bind:this={centerComponent}
      />
    <RightSidebar
      onModeChange={onModeChange}
      onKeyChange={onKeyChange}
      onCapoChange={onCapoChange}
      onFontScaleChange={onFontScaleChange}
      onApiChange={onApiChange}
      onCommunicationChange={onCommunicationChange}
      onMakeCurrentApiOffline={onMakeCurrentApiOffline}
      currentKey={currentKey}
      currentCapo={currentCapo}
      fontScale={fontScale}
      mode={mode}
      wsID={wsID}
      wsConfig={wsConfig}
      visible={rightVisible}
      apiUrl={apiUrl}
      apiPort={apiPort}
      communicationUrl={communicationUrl}
      communicationPort={communicationPort}
      version={version}
      bind:this={rightSidebarComponent}
    />
  </div>
</main>

