<script>
  import { fetchSong } from './api';

  import TitleList from './TitleList.svelte'
  import SetlistList from './SetlistList.svelte'
  import MusicanView from './MusicanView.svelte'

  const getIsMobile = () => window.innerWidth < window.innerHeight;

  let showLeftSidebar = true;
  let showRightSidebar = false;
  let changeSetlist = false;
  let isMobile = getIsMobile();
  let titleListComponent;
  let setlistListComponent;
  let currentSong;

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

  const onSongSelect = async (item) => {
    if (isMobile) {
      showLeftSidebar = false;
    }
    currentSong = await fetchSong(item.title, item.key);
  };
  const onSetlistSelect = async (title) => {
    await titleListComponent.load(title);
    changeSetlist = false;
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
  }
  .left-sidebar-inner {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  #center {
    flex: 2;
    padding-left: 1em;
  }
  #right-sidebar {
    background-color: #0000FF;
    flex: 1;
  }

  #change-setlist, #change-setlist-back {
    width: 100%;
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
      <MusicanView
        song={currentSong}
      />
    </div>
    <div id='right-sidebar' style={!showRightSidebar && "display: none"}></div>
  </div>
</main>

