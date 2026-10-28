<script lang="ts">
  import type { SessionState } from '../api/types';
  import Icon from './Icon.svelte';
  import Pager from './Pager.svelte';
  export let session: SessionState;
  export let selected: string;
  export let selection = false;
  export let onselect: (id: string) => void;
  let page = 0;
  const size = 4;
  $: pages = Math.max(1, Math.ceil(session.categories.length / size));
  $: found = session.categories.filter(c => session.ownFindings.some(f => f.categoryId === c.id)).length;
</script>

<aside class="category-rail panel">
  <div class="sidebar-heading"><h2>{selection ? 'The final cut' : 'Your checklist'}</h2><span class="pill">{found}/{session.categories.length}</span></div>
  <div class="checklist">
    {#each session.categories.slice(page * size, (page + 1) * size) as item, i}
      {@const count = session.ownFindings.filter(f => f.categoryId === item.id).length}
      <button class:active={selected === item.id} onclick={() => onselect(item.id)} aria-pressed={selected === item.id}>
        <span class="category-number" class:has-finding={count > 0}>{#if count}<Icon name="check" size={15}/>{:else}{page * size + i + 1}{/if}</span>
        <span><strong title={item.text}>{item.text}</strong><small>{count ? `${count} saved ${count === 1 ? 'view' : 'views'}${selection ? ' · one selected' : ''}` : selection ? 'No finding · skipped' : 'Nothing spotted. Yet.'}</small></span>
      </button>
    {/each}
  </div>
  <Pager {page} {pages} label="checklist" onpage={value => page = value}/>
  <div class="sidebar-tip"><Icon name={selection ? 'check' : 'info'} size={17}/><p>{selection ? 'No finding? No action needed. Pick your favorites, then mark ready.' : 'One timer. Any order. Choose your final views after time is up.'}</p></div>
</aside>
