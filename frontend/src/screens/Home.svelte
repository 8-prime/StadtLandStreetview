<script lang="ts">
  import Icon from '../lib/components/Icon.svelte';
  import Scene from '../lib/components/Scene.svelte';
  import { mockMode } from '../lib/api';
  export let busy = false;
  export let roomCode = '';
  export let oncreate: (name: string, duration: number, demo: boolean) => void;
  export let onjoin: (code: string, name: string) => void;
  let name = '';
  let code = roomCode;
  let tab: 'create' | 'join' = roomCode ? 'join' : 'create';
  let duration = 600;
  $: if (roomCode) { code = roomCode; tab = 'join'; }
</script>

<main class="home">
  <section class="hero">
    <div class="hero-copy">
      <span class="eyebrow"><span class="tiny-dot"></span> A WORLDWIDE SCAVENGER HUNT</span>
      <h1>Ordinary streets.<br/>Extraordinary<br/><em>little finds.</em></h1>
      <p class="hero-description">A dog on a walk. A very questionable parking job. Explore the world with friends and turn everyday moments into winning discoveries.</p>
      <div class="hero-tags"><span><Icon name="users" size={17}/> Made for friends</span><span><Icon name="compass" size={17}/> Powered by curiosity</span></div>
    </div>
    <div class="hero-art">
      <Scene decorative/>
      <span class="art-sticker sticker-top"><Icon name="pin"/> Somewhere, something good.</span>
      <span class="art-sticker sticker-bottom"><span class="sticker-check"><Icon name="check" size={18}/></span><span><small>FOUND IT!</small>A person walking a dog</span></span>
      <span class="coordinate">52°31′ N &nbsp; 13°24′ E &nbsp; · &nbsp; AN ILLUSTRATED LITTLE DETOUR</span>
    </div>
  </section>

  <section class="home-bottom">
    <div class="how-it-works">
      <span class="eyebrow">THE PLAN IS SIMPLE</span>
      <h2>Three steps.<br/>Countless rabbit holes.</h2>
      <div class="how-steps">
        {#each [{ title: 'Dream it up', text: 'Collect categories with your group.', icon: 'spark' }, { title: 'Go get lost', text: 'Find and frame your Street View discoveries.', icon: 'compass' }, { title: 'Make the call', text: 'Review together. Vote. Crown the most curious.', icon: 'trophy' }] as step, i}
          <div class="how-step"><span class="step-number">0{i + 1}</span><div><strong>{step.title}</strong><p>{step.text}</p></div><Icon name={step.icon}/></div>
        {/each}
      </div>
    </div>
    <div class="entry-card">
      <div class="segmented" role="group" aria-label="Session action">
        <button class:active={tab === 'create'} aria-pressed={tab === 'create'} onclick={() => tab = 'create'}>Start a room</button>
        <button class:active={tab === 'join'} aria-pressed={tab === 'join'} onclick={() => tab = 'join'}>Join friends</button>
      </div>
      <form onsubmit={event => { event.preventDefault(); if (tab === 'create') oncreate(name.trim(), duration, false); else onjoin(code.trim().toUpperCase(), name.trim()); }}>
        <h2>{tab === 'create' ? 'A good time starts here.' : 'Your friends are waiting.'}</h2>
        <p class="muted">{tab === 'create' ? 'Pick a name. Bring your people. Follow your curiosity.' : 'Enter the room code and make yourself at home.'}</p>
        <label for="player-name">Your name</label>
        <input id="player-name" bind:value={name} placeholder="What should we call you?" required maxlength="24" autocomplete="nickname" pattern=".*\S.*"/>
        {#if tab === 'create'}
          <label for="search-duration">Time to explore</label>
          <select id="search-duration" bind:value={duration}><option value={60}>1 minute · a quick test</option><option value={300}>5 minutes · a little wander</option><option value={600}>10 minutes · the sweet spot</option><option value={900}>15 minutes · the scenic route</option><option value={1200}>20 minutes · a proper adventure</option></select>
        {:else}
          <label for="room-code">Room code</label><input id="room-code" bind:value={code} placeholder="ABC123" class="code-input" required maxlength="64" pattern="[A-Za-z0-9_-]+" autocapitalize="characters" spellcheck="false"/>
        {/if}
        <button class="button primary full" disabled={busy}>{busy ? 'One moment…' : tab === 'create' ? 'Let’s make a room' : 'Join the adventure'}<Icon name="arrow"/></button>
      </form>
      {#if mockMode}<div class="entry-demo"><span>Just looking around?</span><button class="text-button" disabled={busy} onclick={() => oncreate(name.trim() || 'Alex', 600, true)}>Try the solo demo <span aria-hidden="true">↗</span></button></div><p class="local-note"><Icon name="info" size={14}/> Local prototype: rooms sync across tabs in this browser. The solo demo includes two simulated friends.</p>{/if}
    </div>
  </section>
  <footer class="home-footer"><span>A little curiosity goes a long way.</span><span>NO SEARCH BAR. NO SHORTCUTS. JUST YOU AND THE WORLD.</span></footer>
</main>
