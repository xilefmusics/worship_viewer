<script>
  import { fetchTitles, fetchSetlist } from '../api';
  import List from './List.svelte';

  export let onSelect;
  let list;


  const load = async (title) => fetchSetlist(title).then((t) => {list.setContent(t.items);onSelect(t.items[0]);});
  const next = () => list.next();
  const prev = () => list.prev();
  const select = (title) => list.select(title);
  const reload = () => fetchTitles().then((t) => {
    let items = t.map((title) => {return {title: title}});
    list.setContent(items);
    onSelect(items[0]);
  });

  export {next, prev, load, select, reload};
</script>

<List
  onSelect={(idx, item) => onSelect(item)}
  bind:this={list}
/>
