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
</script>

<footer class="ready-bar"><div class="ready-people"><div class="avatar-stack">{#each players as p}<span class:person-ready={p.ready} title={`${p.name}: ${p.ready ? 'ready' : p.connected ? 'not ready' : 'disconnected'}`}><Avatar player={p} small/></span>{/each}</div><div><strong>{readyCount} of {players.length} ready</strong><span>{players.some(p => !p.connected) ? 'Waiting for a disconnected spotter to return.' : `All ready? We’ll ${next}.`}</span></div></div><div class="ready-action"><span>{reason || (me.ready ? 'Changed your mind? You can undo until everyone’s ready.' : 'Your crew moves together. Give them the nod.')}</span><button class:ready={me.ready} class="button primary" onclick={onready} disabled={busy || disabled} aria-pressed={me.ready}><Icon name={me.ready ? 'check' : 'arrow'}/>{me.ready ? 'Ready · undo' : 'I’m ready'}</button></div></footer>
