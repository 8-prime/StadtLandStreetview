<script lang="ts">
  import { onMount } from 'svelte';
  import { api, mockMode } from './lib/api';
  import type { Action, Connection, SessionState } from './lib/api/types';
  import Icon from './lib/components/Icon.svelte';
  import ReadyBar from './lib/components/ReadyBar.svelte';
  import Home from './screens/Home.svelte';
  import Lobby from './screens/Lobby.svelte';
  import Categories from './screens/Categories.svelte';
  import Search from './screens/Search.svelte';
  import Selection from './screens/Selection.svelte';
  import Review from './screens/Review.svelte';
  import Results from './screens/Results.svelte';

  let session = $state<SessionState | null>(null);
  let roomCode = $state('');
  let busy = $state(false);
  let loading = $state(false);
  let error = $state('');
  let notice = $state('');
  let connection = $state<Connection>('connecting');
  let showRules = $state(false);
  let leaveDialog: HTMLDialogElement;
  let rulesDialog: HTMLDialogElement;
  let unsubscribe: (() => void) | undefined;
  let routeVersion = 0;
  let noticeTimer: ReturnType<typeof setTimeout>;
  const phases = ['Dream it up', 'Go get lost', 'Make the call'];
  let phaseIndex = $derived(!session || session.phase === 'lobby' ? -1 : session.phase === 'categories' ? 0 : ['searching', 'selection'].includes(session.phase) ? 1 : 2);
  let me = $derived(session?.players.find(p => p.id === session?.meId));
  let canReady = $derived(Boolean(session && connection === 'connected' && !(session.phase === 'categories' && !session.categories.length) && !(session.phase === 'review' && !session.review?.revealed && session.review?.finding.playerId !== session.meId && !session.review?.ownVote)));
  let nextLabel = $derived(session?.phase === 'lobby' ? 'collect categories' : session?.phase === 'categories' ? 'start exploring' : session?.phase === 'selection' ? 'start the review' : session?.review?.revealed ? session.review.index + 1 === session.review.total ? 'see the final scores' : 'see the next finding' : 'reveal the result');

  function toast(message: string) {
    notice = message;
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => notice = '', 3500);
  }
  function receive(value: SessionState) {
    if (session && session.id === value.id && value.revision < session.revision) return;
    const previous = session?.stepId;
    session = value;
    if (previous && previous !== value.stepId) {
      error = '';
      if (value.phase !== 'searching') window.scrollTo({ top: 0, behavior: 'smooth' });
    }
  }
  async function route() {
    const version = ++routeVersion;
    unsubscribe?.();
    unsubscribe = undefined;
    session = null;
    error = '';
    const match = location.hash.match(/^#\/room\/([A-Za-z0-9_-]{1,64})$/);
    roomCode = match?.[1] ?? '';
    window.scrollTo({ top: 0 });
    if (!roomCode) { loading = false; return; }
    loading = true;
    const id = roomCode;
    try {
      const state = await api.get(id);
      if (version !== routeVersion) return;
      receive(state);
      unsubscribe = api.subscribe(id, receive, value => connection = value, message => error = message);
    } catch { /* A shared room link opens the join form until identity is established. */ }
    finally { if (version === routeVersion) loading = false; }
  }
  async function enter(name: string, duration: number, demo: boolean, id?: string) {
    if (!name.trim() || busy) return;
    busy = true; error = '';
    try {
      const room = id || await api.create(name, duration, demo);
      if (id) await api.join(id, name);
      const hash = `#/room/${encodeURIComponent(room)}`;
      if (location.hash === hash) await route(); else location.hash = hash;
    } catch (cause) { error = (cause as Error).message; }
    finally { busy = false; }
  }
  async function act(action: Action): Promise<boolean> {
    if (!session || busy || connection !== 'connected') return false;
    const id = session.id;
    busy = true; error = '';
    try {
      await api.send(id, action);
      const state = await api.get(id);
      if (session?.id === id) receive(state);
      return true;
    } catch (cause) { error = (cause as Error).message; return false; }
    finally { busy = false; }
  }
  async function leave() {
    if (!session || busy) return;
    busy = true;
    try {
      await api.leave(session.id);
      unsubscribe?.(); unsubscribe = undefined;
      leaveDialog.close(); session = null; location.hash = '';
    } catch (cause) { error = (cause as Error).message; leaveDialog.close(); }
    finally { busy = false; }
  }
  async function copyInvite() {
    try { await navigator.clipboard.writeText(location.href); toast('Invite link copied. Bring your people.'); }
    catch { toast(`Room code: ${session?.id}. Copy the link from your address bar.`); }
  }
  onMount(() => {
    void route();
    window.addEventListener('hashchange', route);
    return () => { routeVersion++; unsubscribe?.(); clearTimeout(noticeTimer); window.removeEventListener('hashchange', route); };
  });
</script>

<svelte:head><title>{session ? `${session.id} · ${session.phase === 'review' ? 'Review together' : session.phase === 'searching' ? 'Go get lost' : 'Your adventure'}` : 'Find your kind of extraordinary'} — Stadt Land Streetview</title></svelte:head>

<a class="skip-link" href="#main-content" onclick={event => { event.preventDefault(); document.getElementById('main-content')?.focus(); }}>Skip to content</a>
<header class="site-header"><button class="brand" aria-label="Stadt Land Streetview home" onclick={() => session ? leaveDialog.showModal() : location.hash = ''}><span class="brand-icon"><Icon name="pin" size={27}/></span><span>stadt<span class="brand-dot">.</span>land<span class="brand-dot">.</span><strong>streetview</strong></span></button>
  {#if session}<nav class="phase-nav" aria-label="Game progress">{#each phases as phase, i}<span class:current={phaseIndex === i} class:complete={phaseIndex > i}><span class="nav-number">{#if phaseIndex > i}<Icon name="check" size={12}/>{:else}{i + 1}{/if}</span>{phase}</span>{#if i < 2}<span class="nav-line"></span>{/if}{/each}</nav>{/if}
  <div class="header-actions">{#if session}<button class="room-code" title="Copy invite link" onclick={copyInvite}><span class="tiny-dot"></span> ROOM <strong>{session.id}</strong><Icon name="copy" size={14}/></button><button class="icon-button leave-button" aria-label="Leave room" onclick={() => leaveDialog.showModal()}><Icon name="exit" size={20}/></button>{:else}<span class="header-caption">A little curiosity. A whole world.</span>{/if}<button class="icon-button rules-button" aria-label="How to play" onclick={() => { showRules = true; rulesDialog.showModal(); }}><Icon name="info" size={20}/></button></div>
</header>

{#if session && (mockMode || connection !== 'connected')}<div class="mode-banner" class:connection-warning={connection !== 'connected'}><span class="tiny-dot"></span>{#if connection !== 'connected'}{connection === 'connecting' ? 'Connecting to your room…' : 'Reconnecting… Your place is saved. Actions will resume when connected.'}{:else if session.demo}SOLO DEMO <span>Illustrated scenes · Mika and Jules are simulated peers · no Google Maps requests</span>{:else}LOCAL PROTOTYPE <span>Synced across tabs in this browser only · Axum integration is available via configuration</span>{/if}</div>{/if}
{#if error}<div class="alert" role="alert"><Icon name="info"/><span>{error}</span><button class="icon-button" aria-label="Dismiss error" onclick={() => error = ''}><Icon name="close" size={18}/></button></div>{/if}
{#if notice}<div class="toast" role="status"><Icon name="check" size={18}/>{notice}</div>{/if}

<div id="main-content" tabindex="-1">
{#if loading}<main class="loading-page"><span class="spinner"></span><h2>Finding your people…</h2></main>
{:else if !session}<Home {busy} {roomCode} oncreate={(name, duration, demo) => enter(name, duration, demo)} onjoin={(code, name) => enter(name, 600, false, code)}/>
{:else}<main class="game-main" class:with-ready={!['searching', 'finished'].includes(session.phase)}>
  {#if session.phase === 'lobby'}<Lobby {session} oncopy={copyInvite}/>
  {:else if session.phase === 'categories'}<Categories {session} {busy} onaction={act}/>
  {:else if session.phase === 'searching'}<Search {session} {busy} onaction={act}/>
  {:else if session.phase === 'selection'}<Selection {session} {busy} onaction={act}/>
  {:else if session.phase === 'review' && session.review}<Review {session} {busy} onaction={act}/>
  {:else if session.phase === 'finished'}<Results {session} onhome={() => leaveDialog.showModal()}/>{/if}
  {#if !['searching', 'finished'].includes(session.phase)}<ReadyBar {session} {busy} disabled={!canReady} next={nextLabel} reason={session.phase === 'review' && !canReady ? 'Choose your vote before marking yourself ready.' : ''} onready={() => session && act({ type: 'ready', stepId: session.stepId, ready: !me?.ready })}/>{/if}
</main>{/if}
</div>

<dialog bind:this={leaveDialog} class="modal"><div class="modal-body"><span class="round-icon"><Icon name="exit" size={25}/></span><h2>Taking a different route?</h2><p>Leaving removes you from the readiness requirement. Your submitted findings and earned points stay in the round.</p><div class="modal-actions"><button class="button secondary" onclick={() => leaveDialog.close()}>Stay with the group</button><button class="button primary" disabled={busy} onclick={leave}>Leave room <Icon name="arrow" size={17}/></button></div></div></dialog>
<dialog bind:this={rulesDialog} class="modal rules-modal" onclose={() => showRules = false}>{#if showRules}<div class="modal-body"><button class="icon-button modal-close" aria-label="Close game rules" onclick={() => rulesDialog.close()}><Icon name="close"/></button><span class="eyebrow">A FIELD GUIDE TO A GOOD TIME</span><h2>How to play</h2><ol class="rules-list"><li><strong>Dream it up.</strong><p>Collect categories together. Everyone marks ready to start the search.</p></li><li><strong>Go get lost.</strong><p>Explore Google Maps without search. Save Street View findings for any category. There’s one overall timer.</p></li><li><strong>Pick your favorites.</strong><p>After time expires, choose one saved view per category. The first is selected by default. No finding means no submission.</p></li><li><strong>Make the call.</strong><p>Everyone sees the same finding. Vote on others’ views, then mark ready to reveal. More matches than rejections earns a point; tied votes don’t.</p></li></ol><p class="soft-note">Readiness is reversible until everyone is ready. Mark ready again after each result to move on together. The most accepted matches wins; tied leaders share the win.</p><button class="button primary full" onclick={() => rulesDialog.close()}>Got it. Let’s wander. <Icon name="arrow"/></button></div>{/if}</dialog>
