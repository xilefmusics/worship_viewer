<script>
  export let onSelect;

  let content = [];
  let selectedIdx = 0;

  const selectIdx = (idx, isExtern) => {
    if (idx < 0 || idx >= content.length) {
      return;
    }
    selectedIdx = idx;
    if (!isExtern) {
      onSelect(idx, content[idx]);
    }
  }

  const next = () => selectIdx(selectedIdx + 1);
  const prev = () => selectIdx(selectedIdx - 1);
  const setContent = (c) => {
    content = c;
    selectedIdx = 0;
  }
  const select = (item) => {
    const idx = content.findIndex(elem => elem.title && elem.title === item || elem === item);
    selectIdx(idx, true);
  };

  export {next, prev, setContent, select};
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
    padding: 0.8em;
  }

  li:hover, .selected {
    background-color: #333333;
  }

</style>

<ul>
  {#each content as item, idx}
    <li class={ idx === selectedIdx ? 'selected' : 'not-selected' } on:click={() => selectIdx(idx)}>{item.title === undefined ? item : item.title }</li>
  {/each}
</ul>
