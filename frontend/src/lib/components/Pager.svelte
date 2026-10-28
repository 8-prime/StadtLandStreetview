<script lang="ts">
  import Icon from './Icon.svelte';
  export let page = 0;
  export let pages = 1;
  export let label = 'items';
  export let onpage: (page: number) => void;
</script>

<div class="pager" role="group" aria-label={`${label} pages`}>
  <button class="icon-button" aria-label={`Previous ${label} page`} disabled={page <= 0} onclick={() => onpage(page - 1)}><span class="rotate-arrow"><Icon name="arrow" size={17}/></span></button>
  <span aria-live="polite">{String(page + 1).padStart(2, '0')} <span>/ {String(Math.max(1, pages)).padStart(2, '0')}</span></span>
  <button class="icon-button" aria-label={`Next ${label} page`} disabled={page + 1 >= pages} onclick={() => onpage(page + 1)}><Icon name="arrow" size={17}/></button>
</div>
