<script lang="ts">
  import { onMount } from 'svelte';
  import type { Action, SessionState, Viewpoint } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import Pager from '../lib/components/Pager.svelte';
  import CategoryRail from '../lib/components/CategoryRail.svelte';
  import StreetView from '../lib/maps/StreetView.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  let categoryId = session.categories[0]?.id ?? '';
  let current: Viewpoint | null = null;
  let now = Date.now();
  let savedMessage = '';
  let savedPage = 0;
  let messageTimer: ReturnType<typeof setTimeout>;
  const size = 3;
  const offset = session.serverTime - Date.now();
  $: remaining = Math.max(0, Math.ceil(((session.searchEndsAt ?? 0) - (now + offset)) / 1000));
  $: timer = `${Math.floor(remaining / 60).toString().padStart(2, '0')}:${(remaining % 60).toString().padStart(2, '0')}`;
  $: category = session.categories.find(c => c.id === categoryId);
  $: candidates = session.ownFindings.filter(f => f.categoryId === categoryId);
  $: pages = Math.max(1, Math.ceil(candidates.length / size));
  $: if (savedPage >= pages) savedPage = pages - 1;
  onMount(() => { const interval = setInterval(() => now = Date.now(), 250); return () => { clearInterval(interval); clearTimeout(messageTimer); }; });
  async function save() {
    if (!current || !category) return;
    const nextPage = Math.floor(candidates.length / size);
    if (await onaction({ type: 'save-finding', categoryId, view: current })) {
      savedPage = nextPage;
      savedMessage = 'Evidence secured. Nice spotting.';
      clearTimeout(messageTimer);
      messageTimer = setTimeout(() => savedMessage = '', 3500);
    }
  }
</script>

<section class="phase-heading"><div><span class="eyebrow">02 / HIT THE STREETS</span><h1>EYES ON <span>EVERYTHING.</span></h1></div><div class="search-clock" class:urgent={remaining <= 30}><Icon name="clock" size={23}/><strong aria-label={`${remaining} seconds remaining`}>{timer}</strong><span>{remaining ? 'LEFT TO SPOT' : 'SYNCING…'}</span></div></section>
<div class="search-workspace">
  <CategoryRail {session} selected={categoryId} onselect={id => { categoryId = id; savedPage = 0; savedMessage = ''; }}/>
  <section class="map-workspace panel"><div class="map-topline"><span><Icon name="compass" size={16}/> FIELD OF OPERATIONS</span><span>NO SEARCH BAR. NO EXCUSES.</span></div><div class="view-stage"><div class="view-frame"><StreetView explore demo={session.demo} onview={value => current = value}/></div></div><div class="save-view-bar"><div><span class="eyebrow">CURRENT ASSIGNMENT</span><strong title={category?.text}>{category?.text}</strong><span class="small muted" role="status">{savedMessage || (current ? 'Frame it. Claim it.' : 'Drop Pegman onto a street to start spotting.')}</span></div><button class="button primary" onclick={save} disabled={busy || !current || remaining === 0 || candidates.length >= 12}><Icon name="camera"/> Save view {#if candidates.length}<span class="button-count">{candidates.length}</span>{/if}</button></div></section>
</div>
<section class="saved-strip"><div class="saved-strip-label"><Icon name="camera" size={19}/><strong>Your evidence</strong><span class="small">PRIVATE</span></div><div class="saved-chips">{#each candidates.slice(savedPage * size, (savedPage + 1) * size) as finding, i}<span class="saved-chip"><Icon name="check" size={14}/> View {savedPage * size + i + 1}<button class="icon-button" disabled={busy} aria-label={`Remove saved view ${savedPage * size + i + 1}`} onclick={() => onaction({ type: 'delete-finding', findingId: finding.id })}><Icon name="close" size={13}/></button></span>{:else}<span class="muted small">Your next great find goes here.</span>{/each}</div><Pager page={savedPage} {pages} label="saved views" onpage={value => savedPage = value}/></section>
