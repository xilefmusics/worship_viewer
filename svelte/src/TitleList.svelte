<script>
  import { fetchTitles, fetchSetlist } from './api';
  import List from './List.svelte';

  export let onSelect;
  let list;

  fetchTitles().then((t) => {
    let items = t.map((title) => {return {title: title, key: null}});
    list.setContent(items)
  });

  const load = async (title) => fetchSetlist(title).then((t) => list.setContent(t.items));
  const next = () => list.next();
  const prev = () => list.prev();
  export {next, prev, load};
</script>

<List
  onSelect={(idx, item) => onSelect(item)}
  bind:this={list}
/>
