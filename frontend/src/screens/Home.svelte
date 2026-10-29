<script lang="ts">
  import Icon from '../lib/components/Icon.svelte';
  import GameLogo from '../lib/components/GameLogo.svelte';
  import Spotter from '../lib/components/Spotter.svelte';
  import HomeBackdrop from '../lib/components/HomeBackdrop.svelte';
  import { mockMode } from '../lib/api';
  import type { RoomSettings } from '../lib/api/types';
  export let busy = false;
  export let roomCode = '';
  export let oncreate: (name: string, duration: number, demo: boolean, settings: RoomSettings) => void;
  export let onjoin: (code: string, name: string) => void;
  let name = '';
  let code = roomCode;
  let tab: 'create' | 'join' = roomCode ? 'join' : 'create';
  let duration = 600;
  let proximityEnabled = false;
  $: if (roomCode) { code = roomCode; tab = 'join'; }
</script>

<div class="home-scene">
<HomeBackdrop/>
<main class="home">
  <section class="home-world" aria-label="Stadt Land Streetview">
    <span class="club-label"><Icon name="eye" size={18}/> THE NOT-SO-SECRET SPOTTERS CLUB</span>
    <h1 class="home-title"><GameLogo/></h1>
    <div class="home-tagline"><span>FIND THE WEIRD.</span><span>BRING YOUR FRIENDS.</span></div>
    <div class="home-characters" aria-hidden="true">
      <span class="character-sticker"><Icon name="camera" size={23}/> EVIDENCE, PLEASE.</span>
      <div class="hero-spotter spotter-left"><Spotter seed="club-captain"/></div>
      <div class="hero-spotter spotter-center"><Spotter seed="binocular-buddy" mood="searching"/></div>
      <div class="hero-spotter spotter-right"><Spotter seed="neighborhood-watch" mood="ready"/></div>
      <span class="character-star star-one"><Icon name="spark" size={44}/></span><span class="character-star star-two"><Icon name="spark" size={35}/></span>
    </div>
    <span class="club-footnote">ONE WORLD. SOME VERY QUESTIONABLE PARKING.</span>
  </section>

  <section class="entry-card" aria-label="Play the game">
    <span class="ticket-label"><span class="tiny-dot"></span> SPOTTER CHECK-IN <span>№ 001</span></span>
    <div class="entry-heading"><div><span class="eyebrow">EYES OPEN. GAME ON.</span><h2>{tab === 'create' ? 'Assemble the crew.' : 'Found your people?'}</h2></div><span class="entry-stamp"><Icon name="compass" size={29}/></span></div>
    <div class="segmented" role="group" aria-label="Session action">
      <button class:active={tab === 'create'} aria-pressed={tab === 'create'} onclick={() => tab = 'create'}>Create room</button>
      <button class:active={tab === 'join'} aria-pressed={tab === 'join'} onclick={() => tab = 'join'}>Join room</button>
    </div>
    <form onsubmit={event => { event.preventDefault(); if (tab === 'create') oncreate(name.trim(), duration, false, { proximityEnabled }); else onjoin(code.trim().toUpperCase(), name.trim()); }}>
      <label for="player-name">Your spotter name</label>
      <input id="player-name" bind:value={name} placeholder="An excellent alias, probably" required maxlength="24" autocomplete="nickname" pattern=".*\S.*"/>
      {#if tab === 'create'}
        <label for="search-duration">Time on the streets</label>
        <select id="search-duration" bind:value={duration}><option value={60}>1 minute · test drive</option><option value={300}>5 minutes · quick snoop</option><option value={600}>10 minutes · proper investigation</option><option value={900}>15 minutes · deep dive</option><option value={1200}>20 minutes · leave no street unturned</option></select>
        <label class="room-option"><input type="checkbox" bind:checked={proximityEnabled}/><span><strong>Neighbourhood radio</strong><small>Hear when a spotter is within 5 km.</small></span><span class="option-range" aria-hidden="true">5 km</span></label>
      {:else}
        <label for="room-code">Room code</label><input id="room-code" bind:value={code} placeholder="ABC123" class="code-input" required maxlength="64" pattern="[A-Za-z0-9_-]+" autocapitalize="characters" spellcheck="false"/>
      {/if}
      <button class="button primary full" disabled={busy}>{busy ? 'Checking you in…' : tab === 'create' ? 'Create room' : 'Join room'}<Icon name="arrow"/></button>
    </form>
    {#if mockMode}<div class="entry-demo"><span>Flying solo?</span><button class="text-button" disabled={busy} onclick={() => oncreate(name.trim() || 'Alex', 600, true, { proximityEnabled })}>Try the solo demo <Icon name="arrow" size={15}/></button></div><p class="local-note"><Icon name="info" size={14}/> Local rooms sync across browser tabs. The demo has two simulated spotters.</p>{/if}
    <div class="ticket-tear" aria-hidden="true"></div>
    <div class="entry-rules"><span><b>01</b> MAKE A LIST</span><span><b>02</b> HIT THE STREETS</span><span><b>03</b> JUDGE THE FINDS</span></div>
  </section>
  <footer class="home-footer"><span><Icon name="map" size={16}/> A STREET VIEW SCAVENGER PARTY</span><span>NO SEARCH BAR. JUST SUSPICIOUSLY GOOD OBSERVATION.</span></footer>
</main>
</div>

<style>
  .home-scene { position: relative; height: 100%; isolation: isolate; }
  .home { position: relative; z-index: 1; }
  .room-option { display: flex; align-items: center; gap: 9px; margin: 12px 0 0; padding: 8px; border: 1.5px dashed #202b4555; border-radius: 5px; cursor: pointer; }
  .room-option input { width: 18px; height: 18px; margin: 0; flex-shrink: 0; accent-color: var(--blue); }
  .room-option > span:first-of-type { flex: 1; min-width: 0; }
  .room-option strong { display: block; font-size: 11px; }
  .room-option small { display: block; margin-top: 2px; font-size: 9px; font-weight: 600; color: var(--muted); }
  .option-range { padding: 4px; border: 1.5px solid var(--ink); background: var(--yellow); font-size: 9px; white-space: nowrap; transform: rotate(-4deg); }
  @media (max-height: 780px) { .room-option { margin-top: 8px; padding: 6px; } }
</style>
