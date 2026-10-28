<script lang="ts">
  import type { Player } from '../api/types';
  import type { SpotterMood } from './spotter';
  import Spotter from './Spotter.svelte';
  export let player: Pick<Player, 'id' | 'name' | 'color'> & Partial<Pick<Player, 'ready' | 'connected'>>;
  export let small = false;
  export let large = false;
  export let mood: SpotterMood | undefined = undefined;
  $: expression = mood ?? (player.connected === false ? 'offline' : player.ready ? 'ready' : 'idle');
</script>

<span class="avatar" class:small class:large aria-hidden="true"><Spotter seed={player.id} mood={expression}/></span>
