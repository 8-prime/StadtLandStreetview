<script lang="ts">
  import type { SessionState } from '../lib/api/types';
  import Avatar from '../lib/components/Avatar.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Scene from '../lib/components/Scene.svelte';
  export let session: SessionState;
  export let oncopy: () => void;
</script>

<section class="phase-heading"><div><span class="eyebrow">THE MEETING POINT</span><h1>Good company.<br/><em>Great discoveries.</em></h1><p>Get everyone in the room. Your next adventure starts when you’re all ready.</p></div><span class="phase-note"><Icon name="clock"/>{session.durationSeconds / 60} minutes to explore</span></section>
<div class="lobby-layout">
  <section class="panel lobby-players"><div class="panel-heading"><h2>The curious crew</h2><span class="pill">{session.players.filter(p => !p.left).length} / 12 players</span></div>
    <div class="player-list">{#each session.players.filter(p => !p.left) as player}<div class="player-row"><Avatar {player}/><div><strong>{player.name}{#if player.id === session.meId}<span class="you-label">you</span>{/if}</strong><small>{!player.connected ? 'Reconnecting…' : session.demo && player.id !== session.meId ? 'Simulated friend' : 'Here for the adventure'}</small></div><span class:positive={player.ready} class="status-chip"><Icon name={player.ready ? 'check' : 'clock'} size={14}/>{player.ready ? 'Ready' : 'Getting settled'}</span></div>{/each}</div>
    <div class="invite-box"><span class="round-icon"><Icon name="users"/></span><div><strong>There’s room for your people.</strong><p>{session.demo ? 'Mika and Jules will follow along automatically.' : 'Share the room link, or have friends enter the code.'}</p></div>{#if !session.demo}<button class="button secondary" onclick={oncopy}><Icon name="copy" size={17}/> Copy invite</button>{/if}</div>
  </section>
  <aside class="lobby-aside"><div class="lobby-art"><Scene decorative/><span class="paper-label">A whole world. A shared adventure.</span></div><div class="panel mini-rules"><h3>A few ground rules</h3><p><Icon name="compass" size={18}/> Explore freely. There’s no search bar.</p><p><Icon name="camera" size={18}/> Save a few finds. Pick your favorite later.</p><p><Icon name="users" size={18}/> Everyone votes. The most matches wins.</p></div></aside>
</div>
