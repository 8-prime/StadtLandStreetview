<script lang="ts">
  import Icon from '../lib/components/Icon.svelte';
  import GameLogo from '../lib/components/GameLogo.svelte';
  import Spotter from '../lib/components/Spotter.svelte';
  import HomeBackdrop from '../lib/components/HomeBackdrop.svelte';
  import CountryPicker from '../lib/components/CountryPicker.svelte';
  import DurationPicker from '../lib/components/DurationPicker.svelte';
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
  let bannedCountries: string[] = [];
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
    <form onsubmit={event => { event.preventDefault(); if (tab === 'create') oncreate(name.trim(), duration, false, { proximityEnabled, bannedCountries }); else onjoin(code.trim().toUpperCase(), name.trim()); }}>
      <label for="player-name">Your spotter name</label>
      <input id="player-name" bind:value={name} placeholder="An excellent alias, probably" required maxlength="24" autocomplete="nickname" pattern=".*\S.*"/>
      {#if tab === 'create'}
        <label for="search-duration">Time on the streets</label>
        <DurationPicker id="search-duration" bind:value={duration} disabled={busy}/>
        <label class="room-option"><input type="checkbox" bind:checked={proximityEnabled}/><span><strong>Neighbourhood radio</strong><small>Hear when a spotter is within 5 km.</small></span><span class="option-range" aria-hidden="true">5 km</span></label>
        <CountryPicker bind:selected={bannedCountries} disabled={busy}/>
      {:else}
        <label for="room-code">Room code</label><input id="room-code" bind:value={code} placeholder="ABC123" class="code-input" required maxlength="64" pattern="[A-Za-z0-9_-]+" autocapitalize="characters" spellcheck="false"/>
      {/if}
      <button class="button primary full" disabled={busy}>{busy ? 'Checking you in…' : tab === 'create' ? 'Create room' : 'Join room'}<Icon name="arrow"/></button>
    </form>
    {#if mockMode}<div class="entry-demo"><span>Flying solo?</span><button class="text-button" disabled={busy} onclick={() => oncreate(name.trim() || 'Alex', 600, true, { proximityEnabled, bannedCountries })}>Try the solo demo <Icon name="arrow" size={15}/></button></div><p class="local-note"><Icon name="info" size={14}/> Local rooms sync across browser tabs. The demo has two simulated spotters.</p>{/if}
    <div class="ticket-tear" aria-hidden="true"></div>
    <div class="entry-rules"><span><b>01</b> MAKE A LIST</span><span><b>02</b> HIT THE STREETS</span><span><b>03</b> JUDGE THE FINDS</span></div>
  </section>
  <footer class="home-footer"><span><Icon name="map" size={16}/> A STREET VIEW SCAVENGER PARTY</span><span>NO SEARCH BAR. JUST SUSPICIOUSLY GOOD OBSERVATION.</span></footer>
</main>
</div>

<style>
  .home-scene { position: relative; height: 100%; isolation: isolate; }
  .home { position: relative; z-index: 1; }
  .entry-card .room-option { margin-top: 12px; }
  @media (max-height: 780px) { .entry-card .room-option { margin-top: 8px; } }
</style>
