<script lang="ts">
  import { onDestroy, setContext, type Snippet } from 'svelte';
  import { MapsSession, mapsSessionKey } from './session';

  let { children }: { children: Snippet } = $props();
  const maps = setContext(mapsSessionKey, new MapsSession());
  onDestroy(() => maps.dispose());
</script>

{@render children()}
