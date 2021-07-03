<script>
  import { fetchSong } from './api';
  import { ws, wsID, sendLoadSetlist, sendLoadSong, sendDisplaySection, sendClearBeamer, sendChangeKey, wsConfig } from './websocket';

  import TitleList from './TitleList.svelte'
  import SetlistList from './SetlistList.svelte'
  import MusicanView from './MusicanView.svelte'
  import BeamerControlView from './BeamerControlView.svelte'
  import BeamerView from './BeamerView.svelte'

  const getIsMobile = () => window.innerWidth < window.innerHeight;

  let showLeftSidebar = true;
  let showRightSidebar = true; // TODO only temp
  let changeSetlist = false;
  let isMobile = getIsMobile();
  let titleListComponent;
  let setlistListComponent;
  let beamerViewComponent;
  let currentSong;
  let currentCapo = 0;
  let currentKey = 'Self';
  let fontScale = 0.8;
  let mode = 'musican'; // musican, singer, beamer-control, beamer

  const toggleLeftSidebar = () => showLeftSidebar = !showLeftSidebar;
  const toggleRightSidebar = () => showRightSidebar = !showRightSidebar;

  const onClickCenter = (event) => {
    if (event.y > window.innerHeight*3/4) {
      titleListComponent.next();
    } else if (event.y < window.innerHeight/4) {
      titleListComponent.prev();
    } else if (event.x < window.innerWidth/2) {
      toggleLeftSidebar();
    } else if (event.x > window.innerWidth/2) {
      toggleRightSidebar();
    }
  };

  const onSongSelect = async (item, isRemote) => {
    if (!isRemote) {
      sendLoadSong(item.title, item.key);
      sendClearBeamer();
    }
    if (isMobile) {
      showLeftSidebar = false;
    }
    if (!item.key || item.key === 'Self') {
      item.key = currentKey;
    }
    beamerViewComponent.clear();
    currentSong = await fetchSong(item.title, manipulateKey(item.key, -currentCapo));
  };
  const onSetlistSelect = async (title, isRemote) => {
    if (!isRemote) {
      sendLoadSetlist(title);
    }
    await titleListComponent.load(title);
    changeSetlist = false;
  };
  const onModeChange = (m) => {
    mode = m;
    if (isMobile) {
      showRightSidebar = false;
    }
    if (mode === 'beamer') {
      showRightSidebar = false;
      showLeftSidebar = false;
    }
  };
  const onSectionSelect = (idx, isRemote) => {
    if (idx === null) {
      if (!isRemote) {
        sendClearBeamer()
      }
      beamerViewComponent.clear();
      return;
    }
    if (!isRemote) {
      sendDisplaySection(currentSong.title, idx)
    }
    beamerViewComponent.display(idx);
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
      titleListComponent.select(msg.title);
    } else if (msg.type === "display section") {
      onSectionSelect(msg.idx, true);
    } else if (msg.type === "clear beamer") {
      beamerViewComponent.clear();
    } else if (msg.type === "change key") {
      onKeyChange(msg.key, true);
    }
  });

  document.onkeydown = (e) => {
    if (e.key === " " || e.key === "j" || e.keyCode === 40) {
      titleListComponent.next();
    } else if (e.key === "k" || e.keyCode === 38) {
      titleListComponent.prev();
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
      toggleLeftSidebar();
    } else if (e.keyCode === 39) {
      toggleRightSidebar();
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
  #left-sidebar {
    flex: 1;
    border-right: 4px solid #333333;
  }
  .left-sidebar-inner {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  #center {
    flex: 2;
  }
  #right-sidebar {
    border-left: 4px solid #333333;
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  #change-setlist, #change-setlist-back {
    width: 100%;
  }
  .right-sidebar-item {
    display: flex;
    flex-wrap: wrap;
  }
  .right-sidebar-inneritem {
    padding: 0.8em;
    flex: 1;
  }
</style>

<main>
  <div id='app'>
    <div id='left-sidebar' style={!showLeftSidebar && "display: none"}>
      <div class='left-sidebar-inner' style={changeSetlist && "display: none"}>
        <button id='change-setlist' on:click={() => changeSetlist = true}>Change Setlist</button>
        <TitleList
          onSelect={onSongSelect}
          bind:this={titleListComponent}
        />
      </div>
      <div style={!changeSetlist && "display: none"}>
        <button id='change-setlist-back' on:click={() => changeSetlist = false}>Back</button>
        <SetlistList
          onSelect={onSetlistSelect}
          bind:this={setlistListComponent}
        />
      </div>
    </div>
    <div id='center' style={isMobile && (showLeftSidebar || showRightSidebar) && "display: none"} on:click={onClickCenter}>
      <div style={mode != 'musican' && mode != 'singer' && "display: none"} class='div-fill'>
        <MusicanView
          song={currentSong}
          fontScale={fontScale}
          mode={mode}
        />
      </div>
      <div style={mode != 'beamer-control' && "display: none"} class='div-fill'>
        <BeamerControlView
          song={currentSong}
          onSectionSelect={onSectionSelect}
        />
      </div>
      <div style={mode != 'beamer' && "display: none"} class='div-fill'>
        <BeamerView
          song={currentSong}
          bind:this={beamerViewComponent}
        />
      </div>
    </div>
    <div id='right-sidebar' style={!showRightSidebar && "display: none"}>
      <div id='view-changer-panel' class='right-sidebar-item'>
        <button
          class={`right-sidebar-inneritem ${mode === 'musican' ? 'selected-button': ''}`}
          on:click={() => onModeChange('musican')}
        >Musican</button>
        <button
          class={`right-sidebar-inneritem ${mode === 'singer' ? 'selected-button': ''}`}
          on:click={() => onModeChange('singer')}
        >Singer</button>
        <button
          class={`right-sidebar-inneritem ${mode === 'beamer-control' ? 'selected-button': ''}`}
          on:click={() => onModeChange('beamer-control')}
        >Beamer Control</button>
        <button
          class={`right-sidebar-inneritem ${mode === 'beamer' ? 'selected-button': ''}`}
          on:click={() => onModeChange('beamer')}
        >Beamer</button>
      </div>
      <div class='right-sidebar-item'>
        <button class='right-sidebar-inneritem' on:click={() => onKeyChange('-1')}>-</button>
        <button class='right-sidebar-inneritem' on:click={() => onKeyChange('Self')}>{currentKey}</button>
        <button class='right-sidebar-inneritem' on:click={() => onKeyChange('+1')}>+</button>
      </div>
      <div class='right-sidebar-item'>
        <button class='right-sidebar-inneritem' on:click={() => onCapoChange(-1)}>-</button>
        <button class='right-sidebar-inneritem' on:click={() => onCapoChange(0)}>{currentCapo}</button>
        <button class='right-sidebar-inneritem' on:click={() => onCapoChange(+1)}>+</button>
      </div>
      <div class='right-sidebar-item'>
        <button class='right-sidebar-inneritem' on:click={() => onFontScaleChange('decrement')}>-</button>
        <button class='right-sidebar-inneritem' on:click={() => onFontScaleChange('reset')}>{fontScale}</button>
        <button class='right-sidebar-inneritem' on:click={() => onFontScaleChange('increment')}>+</button>
      </div>
      <div class='right-sidebar-item'>
        <p class='right-sidebar-inneritem'>ID: {wsID}</p>
        <button
          class={`right-sidebar-inneritem ${wsConfig.sendControls ? 'selected-button': ''}`}
          on:click={() => wsConfig.sendControls = !wsConfig.sendControls}
        >Send Controls</button>
        <button
          class={`right-sidebar-inneritem ${wsConfig.receiveControls ? 'selected-button': ''}`}
          on:click={() => wsConfig.receiveControls = !wsConfig.receiveControls}
        >Receive Controls</button>
      </div>
    </div>
  </div>
</main>

