<script lang="ts">
  import type { Action, SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import Pager from '../lib/components/Pager.svelte';
  import CategoryRail from '../lib/components/CategoryRail.svelte';
  import StreetView from '../lib/maps/StreetView.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  let categoryId = session.categories[0]?.id ?? '';
  let page = 0;
  const size = 3;
  $: category = session.categories.find(c => c.id === categoryId);
  $: candidates = session.ownFindings.filter(f => f.categoryId === categoryId);
  $: selected = candidates.find(f => f.id === session.selections[categoryId]);
  $: pages = Math.max(1, Math.ceil(candidates.length / size));
  $: if (page >= pages) page = pages - 1;
</script>

<section class="phase-heading"><div><span class="eyebrow">TIME’S UP / PICK YOUR EVIDENCE</span><h1>YOUR <span>BEST SHOT.</span></h1></div><span class="phase-note"><Icon name="check"/> First saved view selected automatically</span></section>
<div class="selection-layout">
  <CategoryRail {session} selected={categoryId} selection onselect={id => { categoryId = id; page = 0; }}/>
  <section class="selection-detail panel"><div class="selection-title"><h2 title={category?.text}>{category?.text}</h2><span class="pill" class:positive={Boolean(selected)}><Icon name={selected ? 'check' : 'close'} size={14}/>{selected ? 'Final pick' : 'No submission'}</span></div>
    <div class="view-stage">{#if selected}<div class="view-frame">{#key selected.view.demoScene !== undefined ? 'demo' : 'google'}<StreetView view={selected.view}/>{/key}</div>{:else}<div class="empty-list"><Icon name="camera" size={38}/><h3>No finding, no fuss.</h3><p>No saved view for this category. We’ll skip it.</p></div>{/if}</div>
    <div class="candidate-list" role="group" aria-label="Choose final finding">{#each candidates.slice(page * size, (page + 1) * size) as finding, i}<div class:selected={finding.id === selected?.id} class="candidate-row"><button class="candidate-choice" aria-pressed={finding.id === selected?.id} disabled={busy} onclick={() => onaction({ type: 'select', categoryId, findingId: finding.id })}><span class="radio-dot"></span><span><strong>View {page * size + i + 1}</strong><small>{finding.id === selected?.id ? 'Your final submission' : 'Choose this view instead'}</small></span></button><button class="icon-button" disabled={busy} aria-label={`Delete candidate ${page * size + i + 1}`} onclick={() => onaction({ type: 'delete-finding', findingId: finding.id })}><Icon name="trash" size={16}/></button></div>{/each}{#if !candidates.length}<span class="no-candidates">Nothing found here. There’s no extra action to take.</span>{/if}</div>
    <div class="candidate-pagination"><span class="small">{candidates.length} saved {candidates.length === 1 ? 'view' : 'views'} · choose one</span><Pager {page} {pages} label="candidates" onpage={value => page = value}/></div>
  </section>
</div>
