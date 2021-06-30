<script>
  import { fetchSong } from './api';
  import { ws, wsID, sendLoadSetlist, sendLoadSong, sendDisplaySection, sendClearBeamer } from './websocket';

  import TitleList from './TitleList.svelte'
  import SetlistList from './SetlistList.svelte'
  import MusicanView from './MusicanView.svelte'
  import BeamerControlView from './BeamerControlView.svelte'
  import BeamerView from './BeamerView.svelte'

  const getIsMobile = () => window.innerWidth < window.innerHeight;

  let showLeftSidebar = true;
  let showRightSidebar = false;
  let changeSetlist = false;
  let isMobile = getIsMobile();
  let titleListComponent;
  let setlistListComponent;
  let beamerViewComponent;
  let currentSong;
  let mode = 'musican'; // musican, beamer-control, beamer

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
    }
    if (isMobile) {
      showLeftSidebar = false;
    }
    currentSong = await fetchSong(item.title, item.key);
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
    if (!isRemote) {
      sendDisplaySection(currentSong.title, idx)
    }
    beamerViewComponent.display(idx);
  };

  ws.addEventListener("message", (event) => {
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
    }
  });

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
  }
  .left-sidebar-inner {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  #center {
    height: 100%;
    flex: 2;
  }
  #right-sidebar {
    background-color: #0000FF;
    flex: 1;
  }
  #change-setlist, #change-setlist-back {
    width: 100%;
  }
  #view-changer-panel {
    display: flex;
    align-items: stretch;
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
      <div style={mode != 'musican' && "display: none"}>
        <MusicanView
          song={currentSong}
        />
      </div>
      <div style={mode != 'beamer-control' && "display: none"}>
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
      <div id='view-changer-panel'>
        <button on:click={() => onModeChange('musican')}>Musican</button>
        <button on:click={() => onModeChange('beamer-control')}>Beamer Control</button>
        <button on:click={() => onModeChange('beamer')}>Beamer</button>
      </div>
    </div>
  </div>
</main>

