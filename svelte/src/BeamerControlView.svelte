<script>
  export let song;
  export let onSectionSelect;

  const isEmpty = (section) => section.lines.filter((line) => line.text && line.text.length > 0).length === 0;
</script>

<style>
  #main {
    height: 100%;
    overflow-y: auto;
    display: grid;
    padding: 1em;
  }
  .section {
    background-color: #333333;
    padding: 0.8em;
    margin: 0.4em;
    min-width: 20%;
  }
  p {
    padding: 0;
    margin: 0;
    font-size: 0.8em;
  }
  .keyword {
    color: #cc241d;
    font-weight: bold;
    margin-bottom: 0.4em;
  }
  .text {

  }
  @media (min-width: 800px) {
    #main { grid-template-columns: repeat(2, 1fr); }
  }
</style>
<div id='main'>
  {#if song}
    {#each song.sections as section, sidx}
      {#if !isEmpty(section) }
        <div class='section' on:click={(e) => {onSectionSelect(sidx);e.stopPropagation();}}>
          <p class='keyword'>{section.keyword}</p>
          {#each section.lines as line, lidx}
            {#if line.text}
              <p class='text'>{line.text}</p>
            {/if}
          {/each}
        </div>
      {/if}
    {/each}
  {:else}
    <h1>No song loaded!</h1>
  {/if}
</div>
