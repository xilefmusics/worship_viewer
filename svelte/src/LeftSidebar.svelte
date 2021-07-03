<script>
  import TitleList from './TitleList.svelte'
  import SetlistList from './SetlistList.svelte'

  export let onSongSelect;
  export let onSetlistSelect;

  let titleListComponent;
  let setlistListComponent;

  let changeSetlist = false;

  let visible = true;
  const toggle = () => visible = !visible;
  const show = () => visible = true;
  const hide = () => visible = false;

  export {toggle, show, hide, visible};
</script>

<style>
  #main {
    flex: 1;
    border-right: 4px solid #333333;
  }
  .inner {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  #change-setlist, #change-setlist-back {
    width: 100%;
  }
</style>

<div id='main' style={!visible && "display: none"}>
  <div class='inner' style={changeSetlist && "display: none"}>
    <button id='change-setlist' on:click={() => changeSetlist = true}>Change Setlist</button>
    <TitleList
      onSelect={onSongSelect}
      bind:this={titleListComponent}
    />
  </div>
  <div class='inner' style={!changeSetlist && "display: none"}>
    <button id='change-setlist-back' on:click={() => changeSetlist = false}>Back</button>
    <SetlistList
      onSelect={onSetlistSelect}
      bind:this={setlistListComponent}
    />
  </div>
</div>
