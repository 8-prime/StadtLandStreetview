<script lang="ts">
  import type { SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import Pager from '../lib/components/Pager.svelte';
  export let session: SessionState;
  export let onhome: () => void;
  let page = 0;
  const size = 6;
  $: ranked = [...session.players].sort((a, b) => b.score - a.score);
  $: winners = ranked.filter(p => p.score === ranked[0]?.score);
  $: noFindings = session.results.length === 0;
  $: pages = Math.max(1, Math.ceil(ranked.length / size));
</script>

<section class="phase-heading"><div><span class="eyebrow">ALL EVIDENCE EXAMINED</span><h1>CASE <span>CLOSED.</span></h1></div><span class="phase-note"><Icon name="trophy"/> Good eyes. Questionable methods.</span></section>
<div class="results-layout"><section class="winner-poster panel"><span class="winner-ribbon">{noFindings ? 'BETTER LUCK NEXT STREET' : winners.length > 1 ? 'SHARED STREET CRED' : 'TOP SPOTTER'}</span><div class="winner-portrait"><Avatar player={ranked[0]} large mood={noFindings ? 'idle' : 'winner'}/><span class="winner-star"><Icon name="spark" size={55}/></span></div><h2>{noFindings ? 'No finds. Still friends.' : winners.length > 1 ? 'Great minds. Shared glory.' : `${winners[0].name} spotted the lot.`}</h2><p>{noFindings ? 'Nothing submitted this round. A whole world awaits your next attempt.' : winners.length > 3 ? `${winners.length} spotters share the top spot with ${winners[0].score} accepted matches.` : winners.length > 1 ? `${winners.map(p => p.name).join(' & ')} · ${winners[0].score} accepted matches each.` : `${winners[0].score} accepted ${winners[0].score === 1 ? 'match' : 'matches'}. Officially suspiciously observant.`}</p></section>
  <section class="panel scoreboard"><div class="panel-heading"><h2>The final tally</h2><span class="pill">{session.results.length} findings reviewed</span></div><div class="scoreboard-labels"><span>THE SPOTTERS</span><span>MATCHES</span></div><div class="score-rows">{#each ranked.slice(page * size, (page + 1) * size) as player}<div class="score-row" class:winner={!noFindings && player.score === ranked[0].score}><span class="rank">{ranked.findIndex(p => p.score === player.score) + 1}</span><Avatar {player} small/><div><strong title={player.name}>{player.name}{#if player.id === session.meId}<span class="you-label">you</span>{/if}</strong><small>{player.left ? 'LEFT THE ROOM' : player.score === ranked[0].score && !noFindings ? 'EXCEPTIONALLY NOSY' : 'CERTIFIED STREET SPOTTER'}</small></div><span class="score">{player.score}<Icon name={player.score === ranked[0].score && !noFindings ? 'trophy' : 'check'} size={19}/></span></div>{/each}</div><div class="scoreboard-pagination"><span class="small">{session.categories.length} categories investigated</span><Pager {page} {pages} label="scoreboard" onpage={value => page = value}/></div></section>
</div>
<footer class="results-actions"><span><Icon name="eye" size={21}/> YOU’LL NEVER LOOK AT A PARKED CAR THE SAME WAY.</span><button class="button primary" onclick={onhome}>Back to the meeting point <Icon name="arrow"/></button></footer>
