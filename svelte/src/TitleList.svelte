<script>
  import { fetchTitles, fetchSetlist } from './api';
  import List from './List.svelte';

  export let onSelect;
  let list;

  fetchTitles().then((t) => {
    let items = t.map((title) => {return {title: title, key: null}});
    list.setContent(items)
    onSelect(items[0]);
  });

const load = async (title) => fetchSetlist(title).then((t) => {list.setContent(t.items);onSelect(t.items[0]);});
  const next = () => list.next();
  const prev = () => list.prev();
  const select = (title) => list.select(title);
  export {next, prev, load, select};
</script>

<List
  onSelect={(idx, item) => onSelect(item)}
  bind:this={list}
/>
