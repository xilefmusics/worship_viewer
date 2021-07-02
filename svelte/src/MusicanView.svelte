<script>
  export let song;
  export let fontScale;
  export let mode;
</script>

<style>
  #main {
    overflow-y: auto;
    height: 100%;
    padding-left: 1em;
  }
  p {
    padding: 0;
    margin: 0;
    white-space: pre;
    font-family: 'monospace';
    /*font-size: 0.8em;*/
  }
  .keyword {
    padding-top: 0.8em;
    padding-bottom: 0.8em;
    color: #cc241d;
    font-weight: bold;
  }
  .chord {
    color: #98971a;
    font-weight: bold;
  }
  .text {
    color: #98971a;
    padding-bottom: 0.2em;
  }
  .comment {
    color: #458588;
    font-weight: bold;
  }
</style>
<div id='main' style={`font-size: ${fontScale}em`}>
  {#if song}
    {#each song.sections as section, sidx}
      <p class='keyword'>{section.keyword}</p>
      {#each section.lines as line, lidx}
        {#if line.comment}
          <p class='comment'>{line.comment}</p>
        {/if}
        {#if line.chord && mode != 'singer'}
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
