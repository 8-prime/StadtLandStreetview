<script lang="ts">
  import type { Action, SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  let text = '';
  let editing = '';
  let draft = '';
  const suggestions = ['A person walking a dog', 'A car parked illegally', 'A very colorful front door', 'Someone riding a bicycle', 'An unexpected piece of art', 'Something that makes you smile'];
  async function add(value: string) { if (await onaction({ type: 'add-category', text: value })) text = ''; }
</script>

<section class="phase-heading"><div><span class="eyebrow">01 / DREAM IT UP</span><h1>What are we<br/><em>looking for?</em></h1><p>Ordinary, oddly specific, or wonderfully weird. Everyone searches for the same list.</p></div><span class="phase-note"><Icon name="spark"/> Make it findable. Make it fun.</span></section>
<div class="category-layout">
  <section class="panel categories-panel"><div class="panel-heading"><h2>Your scavenger list</h2><span class="pill">{session.categories.length} / 12 categories</span></div>
    {#if !session.categories.length}<div class="empty-list"><Icon name="spark" size={32}/><h3>A blank slate, a whole world.</h3><p>Add the first thing you’d love to stumble upon.</p></div>{/if}
    <ol class="category-list">{#each session.categories as category, i}<li><span class="list-index">{String(i + 1).padStart(2, '0')}</span>{#if editing === category.id}<form class="inline-edit" onsubmit={async event => { event.preventDefault(); if (await onaction({ type: 'edit-category', categoryId: category.id, text: draft })) editing = ''; }}><input bind:value={draft} aria-label="Edit category" maxlength="100" required/><button class="icon-button" aria-label="Save category" disabled={busy}><Icon name="check"/></button><button type="button" class="icon-button" aria-label="Cancel editing" onclick={() => editing = ''}><Icon name="close"/></button></form>{:else}<strong>{category.text}</strong><button class="icon-button" aria-label={`Edit ${category.text}`} disabled={busy} onclick={() => { editing = category.id; draft = category.text; }}><Icon name="edit" size={17}/></button><button class="icon-button" aria-label={`Remove ${category.text}`} disabled={busy} onclick={() => onaction({ type: 'delete-category', categoryId: category.id })}><Icon name="close" size={18}/></button>{/if}</li>{/each}</ol>
    <form class="add-category" onsubmit={event => { event.preventDefault(); void add(text); }}><label class="sr-only" for="category-text">New category</label><input id="category-text" bind:value={text} placeholder="Something worth finding…" required maxlength="100" disabled={session.categories.length >= 12}/><button class="button primary" disabled={busy || !text.trim() || session.categories.length >= 12}><Icon name="plus" size={18}/> Add category</button></form>
    <p class="small muted">Edits reset everyone’s readiness so you can agree on the final list.</p>
  </section>
  <aside class="inspiration"><span class="round-icon"><Icon name="spark" size={26}/></span><span class="eyebrow">A LITTLE INSPIRATION</span><h2>Look for the<br/>little things.</h2><p>The best categories are easy to understand and a little tricky to find.</p><div class="suggestions">{#each suggestions.filter(s => !session.categories.some(c => c.text === s)) as suggestion}<button disabled={busy || session.categories.length >= 12} onclick={() => add(suggestion)}><span>{suggestion}</span><Icon name="plus" size={16}/></button>{/each}</div></aside>
</div>
