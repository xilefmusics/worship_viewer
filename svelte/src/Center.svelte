<script>
  import MusicanView from './MusicanView.svelte'
  import BeamerControlView from './BeamerControlView.svelte'
  import BeamerView from './BeamerView.svelte'

  export let onClickCenter;
  export let onSectionSelect;
  export let mode;
  export let currentSong;
  export let fontScale;
  export let visible;

  let beamerViewComponent;

  const clearBeamer = () => beamerViewComponent.clear();
  const displayBeamer = (idx) => beamerViewComponent.display(idx);

  export {clearBeamer, displayBeamer}

</script>

<style>
  #main {
    flex: 2;
  }
</style>

<div id='main' style={!visible && "display: none"} on:click={onClickCenter}>
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
