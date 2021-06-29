<script>
  export let song;
</script>

<style>
  #main {
    height: 100%;
    overflow-y: auto;
  }
  p {
    padding: 0;
    margin: 0;
    white-space: pre;
    font-family: 'monospace';
    font-size: 0.8em;
  }
  .keyword {
    padding-top: 0.8em;
    padding-bottom: 0.8em;
    color: red;
    font-weight: bold;
  }
  .chord {
    color: green;
    font-weight: bold;
  }
  .text {
    color: green;
    padding-bottom: 0.2em;
  }
</style>
<div id='main'>
  {#if song}
    {#each song.sections as section, sidx}
      <p class='keyword'>{section.keyword}</p>
      {#each section.lines as line, lidx}
        {#if line.chord}
          <p class='chord'>{line.chord}</p>
        {/if}
        {#if line.text}
          <p class='text'>{line.text}</p>
        {/if}
      {/each}
    {/each}
  {:else}
    <h1>No song loaded!</h1>
  {/if}
</div>
