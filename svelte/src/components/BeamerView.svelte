<script>
  export let song;

  let section = null;

const display = (idx) => {
  if (!song) {
    return;
  }
  section = song.sections[idx];
}
  const clear = () => section = null;
  export {display, clear};
</script>

<style>
  #main {
    display: flex;
    justify-content: center;
    padding-bottom: 1.2em;
  }
  #lines {
    position: absolute;
    top: 50%;
    -ms-transform: translateY(-50%);
    transform: translateY(-50%);
  }
  #info {
    text-align: center;
    position: absolute;
    bottom: 0.5em;
    right: 0.5em;
  }
  .line {
    text-align: center;
    font-size: 2em;
    margin: 0.2em;
    padding: 0;
  }
</style>

<div id='main' class='div-fill'>
  <div id='lines'>
    {#if song && section}
      {#each section.lines as line, idx}
        {#if line.text}
          <p class='line'>{line.text}</p>
        {/if}
      {/each}
    {/if}
  </div>
  <div id='info'>
    {#if song && section}
      {song.title}
    {/if}
  </div>
</div>
