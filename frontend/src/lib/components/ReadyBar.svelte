<script lang="ts">
  import type { SessionState } from '../api/types';
  import Avatar from './Avatar.svelte';
  import Icon from './Icon.svelte';
  export let session: SessionState;
  export let busy = false;
  export let disabled = false;
  export let reason = '';
  export let next = 'continue';
  export let onready: () => void;
  $: me = session.players.find(p => p.id === session.meId)!;
  $: players = session.players.filter(p => !p.left);
  $: readyCount = players.filter(p => p.ready).length;
  $: voting = session.phase === 'review' && session.review?.finding.playerId !== session.meId;
  $: searching = session.phase === 'searching';
</script>

<footer class="ready-bar"><div class="ready-people"><div class="avatar-stack">{#each players as p}<span class:person-ready={p.ready} title={`${p.name}: ${p.ready ? 'ready' : p.connected ? 'not ready' : 'disconnected'}`}><Avatar player={p} small/></span>{/each}</div><div><strong>{readyCount} of {players.length} ready</strong><span>{players.some(p => !p.connected && !p.ready) ? 'Waiting for a disconnected spotter to return.' : `All ready? We’ll ${next}.`}</span></div></div><div class="ready-action"><span role="status">{voting ? session.review?.ownVote ? 'Vote cast. Waiting for the crew — no ready click needed.' : 'Cast your vote. It counts as ready automatically.' : reason || (me.ready ? 'Changed your mind? You can undo until everyone’s ready.' : searching ? 'Finish early when everyone is done. The timer keeps running.' : 'Your crew moves together. Give them the nod.')}</span>{#if !voting}<button class:ready={me.ready} class="button primary" onclick={onready} disabled={busy || disabled} aria-pressed={me.ready}><Icon name={me.ready ? 'check' : 'arrow'}/>{me.ready ? 'Ready · undo' : searching ? 'Done searching' : 'I’m ready'}</button>{/if}</div></footer>
