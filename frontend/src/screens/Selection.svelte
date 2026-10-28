<script lang="ts">
  import type { Action, SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import StreetView from '../lib/maps/StreetView.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  let categoryId = session.categories[0]?.id ?? '';
  $: category = session.categories.find(c => c.id === categoryId);
  $: candidates = session.ownFindings.filter(f => f.categoryId === categoryId);
  $: selected = candidates.find(f => f.id === session.selections[categoryId]);
</script>

<section class="phase-heading compact"><div><span class="eyebrow">THE FINAL CUT</span><h1>A few good finds.<br/><em>Your best shot.</em></h1><p>Time’s up. Pick the view you want the group to judge for each category.</p></div><span class="phase-note"><Icon name="check"/> First saved view selected automatically</span></section>
<div class="selection-layout">
  <aside class="panel selection-sidebar"><div class="panel-heading"><h2>Your categories</h2></div><div class="checklist">{#each session.categories as item, i}{@const count = session.ownFindings.filter(f => f.categoryId === item.id).length}<button class:active={categoryId === item.id} onclick={() => categoryId = item.id}><span class="category-number" class:has-finding={count > 0}>{#if count}<Icon name="check" size={15}/>{:else}{i + 1}{/if}</span><span><strong>{item.text}</strong><small>{count ? `${count} ${count === 1 ? 'candidate' : 'candidates'} · one selected` : 'No finding · automatically skipped'}</small></span></button>{/each}</div><p class="sidebar-tip">Nothing found for a category? No action needed. It simply won’t be submitted.</p></aside>
  <section class="selection-detail"><div class="panel-heading"><div><span class="eyebrow">FINAL SUBMISSION</span><h2>{category?.text}</h2></div>{#if selected}<span class="pill positive"><Icon name="check" size={14}/> Selected</span>{/if}</div><div class="selection-preview">{#if selected}{#key selected.view.demoScene !== undefined ? 'demo' : 'google'}<StreetView view={selected.view}/>{/key}{:else}<div class="empty-list"><Icon name="camera" size={38}/><h3>No finding, no fuss.</h3><p>You didn’t save a view for this category. It will be skipped during review.</p></div>{/if}</div><div class="candidate-list" role="group" aria-label="Choose final finding">{#each candidates as finding, i}<div class:selected={finding.id === selected?.id} class="candidate-row"><button class="candidate-choice" aria-pressed={finding.id === selected?.id} disabled={busy} onclick={() => onaction({ type: 'select', categoryId, findingId: finding.id })}><span class="radio-dot"></span><span><strong>View {i + 1}</strong><small>{finding.id === selected?.id ? 'Your final submission' : 'Choose this view instead'}</small></span><span class="view-coordinates">{finding.view.demoScene !== undefined ? 'Illustrated demo' : `${finding.view.position.lat.toFixed(4)}°, ${finding.view.position.lng.toFixed(4)}°`}</span></button><button class="icon-button" disabled={busy} aria-label={`Delete candidate ${i + 1}`} onclick={() => onaction({ type: 'delete-finding', findingId: finding.id })}><Icon name="trash" size={17}/></button></div>{/each}</div></section>
</div>
