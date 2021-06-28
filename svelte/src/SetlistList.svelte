<script>
  import { fetchSetlists } from './api';
  import List from './List.svelte';

  export let onSelect;
  let list;

  fetchSetlists().then((t) => list.setContent(t));

  const next = () => list.next();
  const prev = () => list.prev();
  export {next, prev};
</script>

<List
  onSelect={(idx, item) => onSelect(item)}
  bind:this={list}
/>
