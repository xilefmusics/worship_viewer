<script>
  import { fetchSetlists } from '../api';
  import List from './List.svelte';

  export let onSelect;
  let list;

  const next = () => list.next();
  const prev = () => list.prev();
  const reload = () => fetchSetlists().then((t) => list.setContent(t));

  reload();
  export {next, prev, reload};
</script>

<List
  onSelect={(idx, item) => onSelect(item)}
  bind:this={list}
/>
