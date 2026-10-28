<script lang="ts">
  import { onMount } from 'svelte';
  import type { Action, SessionState, Viewpoint } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import StreetView from '../lib/maps/StreetView.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  let categoryId = session.categories[0]?.id ?? '';
  let current: Viewpoint | null = null;
  let now = Date.now();
  let savedMessage = '';
  let messageTimer: ReturnType<typeof setTimeout>;
  // Capture the server clock offset once; do not stretch the countdown on rerenders.
  const offset = session.serverTime - Date.now();
  $: remaining = Math.max(0, Math.ceil(((session.searchEndsAt ?? 0) - (now + offset)) / 1000));
  $: timer = `${Math.floor(remaining / 60).toString().padStart(2, '0')}:${(remaining % 60).toString().padStart(2, '0')}`;
  $: category = session.categories.find(c => c.id === categoryId);
  $: candidates = session.ownFindings.filter(f => f.categoryId === categoryId);
  $: foundCount = session.categories.filter(c => session.ownFindings.some(f => f.categoryId === c.id)).length;
  onMount(() => { const interval = setInterval(() => now = Date.now(), 250); return () => { clearInterval(interval); clearTimeout(messageTimer); }; });
  async function save() {
    if (!current || !category) return;
    if (await onaction({ type: 'save-finding', categoryId, view: current })) {
      savedMessage = `Saved for “${category.text}”`;
      clearTimeout(messageTimer);
      messageTimer = setTimeout(() => savedMessage = '', 3500);
    }
  }
</script>

<section class="phase-heading compact"><div><span class="eyebrow">02 / GO GET LOST</span><h1>The world is <em>your playground.</em></h1><p>Choose a category. Find a moment. Save the view.</p></div><div class="search-clock" class:urgent={remaining <= 30}><Icon name="clock" size={25}/><div><strong aria-label={`${remaining} seconds remaining`}>{timer}</strong><span>{remaining ? 'LEFT TO EXPLORE' : 'WAITING FOR PHASE UPDATE'}</span></div></div></section>
<div class="search-workspace">
  <aside class="search-sidebar"><div class="sidebar-heading"><span class="eyebrow">YOUR CHECKLIST</span><span>{foundCount}/{session.categories.length}</span></div><div class="checklist">{#each session.categories as item, i}{@const count = session.ownFindings.filter(f => f.categoryId === item.id).length}<button class:active={categoryId === item.id} onclick={() => categoryId = item.id}><span class="category-number" class:has-finding={count > 0}>{#if count}<Icon name="check" size={15}/>{:else}{i + 1}{/if}</span><span><strong>{item.text}</strong><small>{count ? `${count} saved ${count === 1 ? 'view' : 'views'}` : 'Your next discovery awaits'}</small></span>{#if categoryId === item.id}<Icon name="arrow" size={17}/>{/if}</button>{/each}</div><div class="sidebar-tip"><Icon name="info" size={18}/><p>One timer for the whole list. Save multiple views and choose your favorites after time is up.</p></div></aside>
  <section class="map-workspace"><div class="map-topline"><span><Icon name="compass" size={17}/> FREE TO WANDER</span><span>No search. Just exploration.</span></div><div class="explore-map"><StreetView explore demo={session.demo} onview={value => current = value}/></div><div class="save-view-bar"><div><span class="eyebrow">SAVE THIS VIEW FOR</span><strong>{category?.text}</strong><span class="small muted">{savedMessage || (current ? 'Your current framing will be preserved.' : 'Enter Street View to save a finding.')}</span></div><button class="button primary" onclick={save} disabled={busy || !current || remaining === 0 || candidates.length >= 12}><Icon name="camera"/> Save view {#if candidates.length}<span class="button-count">{candidates.length}</span>{/if}</button></div></section>
</div>
<section class="saved-strip"><div><Icon name="camera" size={18}/><strong>Saved for this category</strong><span class="muted small">Only you can see these.</span></div><div class="saved-chips">{#each candidates as finding, i}<span class="saved-chip"><Icon name="check" size={14}/> View {i + 1}<button class="icon-button" disabled={busy} aria-label={`Remove saved view ${i + 1}`} onclick={() => onaction({ type: 'delete-finding', findingId: finding.id })}><Icon name="close" size={13}/></button></span>{:else}<span class="muted small">Nothing saved yet. A good find might be just around the corner.</span>{/each}</div>{#if session.demo}<button class="text-button demo-shortcut" disabled={busy} onclick={() => onaction({ type: 'demo-end-search' })}>Demo: end search <span aria-hidden="true">→</span></button>{/if}</section>
