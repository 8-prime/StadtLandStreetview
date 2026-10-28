<script lang="ts">
  import Icon from './Icon.svelte';
  export let compact = false;
</script>

<span class="game-logo" class:compact aria-label="Stadt Land Streetview">
  <span class="logo-sign sign-stadt">STADT<span class="sign-arrow sign-diagonal"><Icon name="arrow" size={50}/></span></span>
  <span class="logo-sign sign-land">LAND<span class="sign-bolt"><Icon name="spark" size={48}/></span></span>
  <span class="logo-sign sign-street">STREETVIEW<span class="sign-arrow"><Icon name="arrow" size={42}/></span></span>
</span>
