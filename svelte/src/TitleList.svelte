<script>
  import { onMount } from 'svelte';
  import { fetchTitles } from './api';

  export let onSongSelect;

  let selectedIdx = 0;
  let titles;
  fetchTitles().then((t) => titles = t);

  const selectIdx = (idx) => {
    if (idx < 0 || idx >= titles.length) {
      return;
    }
    selectedIdx = idx;
    onSongSelect(titles[idx]);
  }

  const next = () => selectIdx(selectedIdx + 1);
  const prev = () => selectIdx(selectedIdx - 1);

  export {next, prev};
</script>

<style>
  ul {
    overflow-y: auto;
    height: 100%;
    list-style-type: none;
    padding: 0;
    margin: 0;
  }

  li {
    background-color: #222222;
    color: #DDDDDD;
    padding: 0.8em;
    margin-left: 0.8em;
    margin-right: 0.8em;
  }

  li:hover, .selected {
    background-color: #333333;
  }

</style>

{#if titles != undefined}
  <ul>
    {#each titles as title, i}
      {#if i === selectedIdx}
        <li class='selected' on:click={() => selectIdx(i)}>{title}</li>
      {:else}
        <li on:click={() => selectIdx(i)}>{title}</li>
      {/if}
    {/each}
  </ul>
{/if}
