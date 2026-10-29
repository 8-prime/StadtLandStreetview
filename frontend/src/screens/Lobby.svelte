<script lang="ts">
  import type { SessionState } from '../lib/api/types';
  import Avatar from '../lib/components/Avatar.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Spotter from '../lib/components/Spotter.svelte';
  export let session: SessionState;
  export let oncopy: () => void;
  $: players = session.players.filter(p => !p.left);
</script>

<section class="phase-heading"><div><span class="eyebrow">THE MEETING POINT</span><h1>THE USUAL <span>SUSPECTS.</span></h1></div><span class="phase-note"><Icon name="clock"/>{session.durationSeconds / 60} minutes on the streets</span></section>
<div class="lobby-layout">
  <section class="panel lobby-players">
    <div class="panel-heading"><h2>The curious crew</h2><span class="pill">{players.length} / 12 spotters</span></div>
    <div class="player-grid" class:dense={players.length > 8}>
      {#each players as player}<div class="player-card" class:is-ready={player.ready} class:is-offline={!player.connected}>
        <span class="player-card-status"><Icon name={!player.connected ? 'info' : player.ready ? 'check' : 'clock'} size={14}/>{!player.connected ? 'Disconnected' : player.ready ? 'Ready!' : 'Getting settled'}</span>
        <Avatar {player} large/>
        <strong title={player.name}>{player.name}{#if player.id === session.meId}<span class="you-label">you</span>{/if}</strong>
        <small>{session.demo && player.id !== session.meId ? 'SIMULATED SPOTTER' : 'LICENSED TO LOOK'}</small>
      </div>{/each}
      {#if players.length < 4}<button class="empty-player" onclick={oncopy} disabled={session.demo}><Icon name="plus" size={28}/><strong>{session.demo ? 'Your crew is here.' : 'Missing an accomplice?'}</strong><span>{session.demo ? 'Mika & Jules play automatically.' : 'Copy an invite link'}</span></button>{/if}
    </div>
    <div class="invite-box"><Icon name="users" size={20}/><p>{session.demo ? 'Your simulated crew is ready when you are.' : 'Get everyone in. Then everyone gets a say.'}</p>{#if !session.demo}<button class="button secondary" onclick={oncopy}><Icon name="copy" size={16}/> Copy invite</button>{/if}</div>
  </section>
  <aside class="briefing panel"><span class="briefing-tab">FIELD BRIEFING</span><div class="briefing-character"><Spotter seed="dispatch-chief" mood="searching"/></div><h2>A perfectly normal<br/>amount of nosy.</h2><ol><li><span>01</span><div><strong>Make the list.</strong><p>Pick things worth spotting.</p></div></li><li><span>02</span><div><strong>Get out there.</strong><p>Save your Street View evidence.</p></div></li><li><span>03</span><div><strong>Make the call.</strong><p>Most accepted matches wins.</p></div></li></ol><span class="briefing-bottom">{session.settings.proximityEnabled ? 'NEIGHBOURHOOD RADIO ON · 5 KM' : 'NO SEARCH. NO SHORTCUTS.'}</span></aside>
</div>
