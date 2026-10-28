<script lang="ts">
  import type { Player } from '../api/types';
  export let player: Pick<Player, 'name' | 'color'>;
  export let small = false;
</script>
<span class="avatar color-{player.color % 6}" class:small aria-hidden="true">{player.name.slice(0, 1).toUpperCase()}</span>
