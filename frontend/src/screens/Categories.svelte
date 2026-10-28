<script lang="ts">
  import type { Action, SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import Pager from '../lib/components/Pager.svelte';
  import Spotter from '../lib/components/Spotter.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  let text = '';
  let editing = '';
  let draft = '';
  let page = 0;
  const size = 4;
  const suggestions = ['A person walking a dog', 'A car parked illegally', 'A very colorful front door', 'Someone riding a bicycle', 'An unexpected piece of art', 'Something that makes you smile'];
  $: pages = Math.max(1, Math.ceil(session.categories.length / size));
  $: if (page >= pages) page = pages - 1;
  async function add(value: string) {
    const nextPage = Math.floor(session.categories.length / size);
    if (await onaction({ type: 'add-category', text: value })) { text = ''; page = nextPage; }
  }
</script>

<section class="phase-heading"><div><span class="eyebrow">01 / MAKE A LIST</span><h1>WANTED: <span>WEIRD STUFF.</span></h1></div><span class="phase-note"><Icon name="spark"/> Findable. Debatable. Slightly ridiculous.</span></section>
<div class="category-layout">
  <section class="panel categories-panel">
    <div class="panel-heading"><h2>Your scavenger list</h2><span class="pill">{session.categories.length} / 12 assignments</span></div>
    <div class="category-list-area">
      {#if !session.categories.length}<div class="empty-list"><Icon name="eye" size={42}/><h3>What looks suspiciously findable?</h3><p>Add a category. Everyone will be on the same case.</p></div>{/if}
      <ol class="category-list" start={page * size + 1}>
        {#each session.categories.slice(page * size, (page + 1) * size) as category, i}
          <li><span class="list-index">{String(page * size + i + 1).padStart(2, '0')}</span>
            {#if editing === category.id}<form class="inline-edit" onsubmit={async event => { event.preventDefault(); if (await onaction({ type: 'edit-category', categoryId: category.id, text: draft })) editing = ''; }}><input bind:value={draft} aria-label="Edit category" maxlength="100" required/><button class="icon-button" aria-label="Save category" disabled={busy}><Icon name="check"/></button><button type="button" class="icon-button" aria-label="Cancel editing" onclick={() => editing = ''}><Icon name="close"/></button></form>
            {:else}<strong title={category.text}>{category.text}</strong><button class="icon-button" aria-label={`Edit ${category.text}`} disabled={busy} onclick={() => { editing = category.id; draft = category.text; }}><Icon name="edit" size={17}/></button><button class="icon-button" aria-label={`Remove ${category.text}`} disabled={busy} onclick={() => onaction({ type: 'delete-category', categoryId: category.id })}><Icon name="close" size={18}/></button>{/if}
          </li>
        {/each}
      </ol>
    </div>
    <div class="list-pagination"><span class="small">Edits reset everyone’s readiness.</span><Pager {page} {pages} label="categories" onpage={value => { page = value; editing = ''; }}/></div>
    <form class="add-category" onsubmit={event => { event.preventDefault(); void add(text); }}><label class="sr-only" for="category-text">New category</label><input id="category-text" bind:value={text} placeholder="A very questionable parking job…" required maxlength="100" disabled={session.categories.length >= 12}/><button class="button primary" disabled={busy || !text.trim() || session.categories.length >= 12}><Icon name="plus" size={18}/> Add category</button></form>
  </section>
  <aside class="inspiration panel"><div class="inspiration-top"><div><span class="eyebrow">STUCK? STEAL THESE.</span><h2>A few leads.</h2></div><div class="inspiration-character"><Spotter seed="idea-detective"/></div></div><p>Specific enough to judge.<br/>Weird enough to be worth it.</p><div class="suggestions">{#each suggestions.filter(s => !session.categories.some(c => c.text === s)).slice(0, 4) as suggestion}<button disabled={busy || session.categories.length >= 12} onclick={() => add(suggestion)}><span>{suggestion}</span><Icon name="plus" size={17}/></button>{/each}</div><span class="inspiration-note"><Icon name="info" size={16}/> EVERYONE SEARCHES FOR THE SAME LIST.</span></aside>
</div>
