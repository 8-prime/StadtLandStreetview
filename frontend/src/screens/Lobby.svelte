<script lang="ts">
  import type { Action, SessionState } from '../lib/api/types';
  import { formatDuration } from '../lib/api/duration';
  import Avatar from '../lib/components/Avatar.svelte';
  import CountryPicker from '../lib/components/CountryPicker.svelte';
  import DurationPicker from '../lib/components/DurationPicker.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Spotter from '../lib/components/Spotter.svelte';
  export let session: SessionState;
  export let busy = false;
  export let connected = true;
  export let oncopy: () => void;
  export let onaction: (action: Action) => Promise<boolean>;
  $: players = session.players.filter(p => !p.left);
  $: banned = session.settings.bannedCountries ?? [];
  $: locked = busy || !connected;
  const update = (change: Omit<Extract<Action, { type: 'update-settings' }>, 'type'>) => onaction({ type: 'update-settings', ...change });
</script>

<section class="phase-heading"><div><span class="eyebrow">THE MEETING POINT</span><h1>THE USUAL <span>SUSPECTS.</span></h1></div><span class="phase-note"><Icon name="clock"/>{formatDuration(session.durationSeconds)} on the streets</span></section>
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
  <aside class="briefing panel room-rules" aria-labelledby="room-rules-title">
    <span class="briefing-tab">ROOM RULES</span>
    <div class="briefing-character"><Spotter seed="dispatch-chief" mood="searching"/></div>
    <h2 id="room-rules-title">House rules,<br/>agreed together.</h2>
    <div class="rules-controls">
      <div class="rules-duration">
        <label for="lobby-duration">Time on the streets</label>
        <DurationPicker id="lobby-duration" value={session.durationSeconds} disabled={locked} onchange={durationSeconds => update({ durationSeconds })}/>
      </div>
      <label class="room-option"><input type="checkbox" checked={session.settings.proximityEnabled} disabled={locked} onchange={event => update({ proximityEnabled: event.currentTarget.checked })}/><span><strong>Neighbourhood radio</strong><small>Hear when a spotter is within 5 km.</small></span><span class="option-range" aria-hidden="true">5 km</span></label>
      <CountryPicker selected={banned} disabled={locked} onchange={bannedCountries => update({ bannedCountries })}/>
    </div>
    <span class="briefing-bottom">ANYONE CAN EDIT · CHANGES RESET READY</span>
  </aside>
</div>

<style>
  .room-rules h2 { margin-bottom: 12px; }
  .rules-controls { display: flex; flex-direction: column; gap: 8px; width: 100%; }
  .rules-controls label:not(.room-option) { margin: 0 0 5px; font-size: 11px; }
  .rules-controls :global(select), .rules-controls :global(input:not([type='checkbox'])) { height: 40px; font-size: 12px; }
  .rules-controls :global(.ban-option) { margin: 0; }
  @media (max-height: 780px) {
    .room-rules .briefing-character { flex-basis: 90px; }
    .room-rules h2 { margin-bottom: 8px; }
    .rules-controls { gap: 6px; }
    .rules-controls :global(select), .rules-controls :global(input:not([type='checkbox'])) { height: 36px; }
  }
  @media (max-height: 650px) { .room-rules .briefing-character, .room-rules h2 { display: none; } }
  /* The briefing is hidden on narrow screens, but the rules must stay editable. */
  @media (max-width: 900px) {
    .lobby-layout { grid-template-rows: minmax(0, 1fr) auto; }
    .room-rules { display: block; padding: 22px 14px 12px; }
    .room-rules .briefing-character, .room-rules h2, .room-rules .briefing-bottom { display: none; }
    .rules-controls { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); align-items: end; }
  }
</style>
