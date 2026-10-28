<script lang="ts">
  import type { SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  export let session: SessionState;
  export let onhome: () => void;
  $: ranked = [...session.players].sort((a, b) => b.score - a.score);
  $: winners = ranked.filter(p => p.score === ranked[0]?.score);
  $: noFindings = session.results.length === 0;
</script>

<section class="results-header"><span class="trophy-medallion"><Icon name="trophy" size={42}/></span><span class="eyebrow">THE SCENIC ROUTE TO VICTORY</span><h1>{noFindings ? 'No finds. Still friends.' : winners.length > 1 ? 'Great minds. Shared glory.' : `${winners[0].name} takes the scenic crown.`}</h1><p>{noFindings ? 'No final findings were submitted this round. The world will still be there for another try.' : winners.length > 1 ? `${winners.map(p => p.name).join(' & ')} finish tied with ${winners[0].score} accepted matches each.` : `${winners[0].score} accepted ${winners[0].score === 1 ? 'match' : 'matches'}. A well-earned win for a curious mind.`}</p></section>
<section class="panel scoreboard"><div class="panel-heading"><h2>The final tally</h2><span class="pill">{session.categories.length} categories · {session.results.length} findings reviewed</span></div><div class="scoreboard-labels"><span>EXPLORER</span><span>ACCEPTED MATCHES</span></div>{#each ranked as player, i}<div class="score-row" class:winner={!noFindings && player.score === ranked[0].score}><span class="rank">{i > 0 && player.score === ranked[i - 1].score ? '=' : ranked.findIndex(p => p.score === player.score) + 1}</span><Avatar {player}/><div><strong>{player.name}{#if player.id === session.meId}<span class="you-label">you</span>{/if}</strong><small>{player.left ? 'Left the room' : player.score === ranked[0].score && !noFindings ? 'A world-class wandering eye' : 'A fellow finder of good things'}</small></div><span class="score">{player.score}<Icon name={player.score === ranked[0].score && !noFindings ? 'trophy' : 'check'} size={20}/></span></div>{/each}<div class="scoreboard-footer"><span><Icon name="spark" size={18}/> The best part? All the things you didn’t expect to find.</span><button class="button primary" onclick={onhome}>Back to the meeting point <Icon name="arrow"/></button></div></section>
