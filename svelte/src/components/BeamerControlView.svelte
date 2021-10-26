<script>
  export let song;
  export let onSectionSelect;
  export let translation;

  const isEmpty = (section) => section.lines.filter((line) => line.text && line.text.length > 0).length === 0;
</script>

<style>
  #main {
    height: 100%;
    overflow-y: auto;
  }
  #grid {
    margin: 0.8em;
    display: grid;
  }
  #clear {
    font-size: 0.8em;
    margin: 0 1.6em;
    text-align: center;
    color: #cc241d;
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
  h1 {
    text-align: center;
  }
  @media (min-width: 800px) {
    #grid { grid-template-columns: repeat(2, 1fr); }
  }
</style>
<div id='main'>
  {#if song}
    <h1>{song.title}</h1>
    <div
     class='section'
     id='clear'
     on:click={(e) => {
       onSectionSelect(null);
      e.stopPropagation();
    }}>
     Clear Screen
    </div>
    <div id='grid'>
      {#each song.sections as section, sidx}
        {#if !isEmpty(section) }
          <div class='section' on:click={(e) => {onSectionSelect(sidx);e.stopPropagation();}}>
            <p class='keyword'>{section.keyword}</p>
            {#each section.lines as line, lidx}
              {#if !translation && line.text || translation && line.text && !line.translation_text}
                <p class='text'>{line.text}</p>
              {/if}
              {#if translation && line.translation_text}
                <p class='text'>{line.translation_text}</p>
              {/if}
            {/each}
          </div>
        {/if}
      {/each}
    </div>
  {:else}
    <h1>No song loaded!</h1>
  {/if}
</div>
