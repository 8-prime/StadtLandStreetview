<script lang="ts">
  import { onMount } from 'svelte';
  import { api, mockMode } from './lib/api';
  import type { Action, Connection, SessionState } from './lib/api/types';
  import Icon from './lib/components/Icon.svelte';
  import ReadyBar from './lib/components/ReadyBar.svelte';
  import GameLogo from './lib/components/GameLogo.svelte';
  import Spotter from './lib/components/Spotter.svelte';
  import Pager from './lib/components/Pager.svelte';
  import Home from './screens/Home.svelte';
  import Lobby from './screens/Lobby.svelte';
  import Categories from './screens/Categories.svelte';
  import Search from './screens/Search.svelte';
  import Selection from './screens/Selection.svelte';
  import Review from './screens/Review.svelte';
  import Results from './screens/Results.svelte';
  import MapsSession from './lib/maps/MapsSession.svelte';

  let session = $state<SessionState | null>(null);
  let roomCode = $state('');
  let busy = $state(false);
  let loading = $state(false);
  let error = $state('');
  let notice = $state('');
  let connection = $state<Connection>('connecting');
  let showRules = $state(false);
  let rulesPage = $state(0);
  const rules = [
    { title: 'Make a list.', text: 'Collect things to find with your crew. A dog walker? A questionable parking job? Everyone marks ready to start the same scavenger hunt.', mood: 'idle' as const },
    { title: 'Hit the streets.', text: 'Explore Google Maps without a search bar. Save Street View evidence for any category. One timer covers the whole list, so spend it how you like.', mood: 'searching' as const },
    { title: 'Pick your evidence.', text: 'When time expires, choose one saved view per category. Your first is selected automatically. No finding means no submission. Mark ready when you’re happy.', mood: 'ready' as const },
    { title: 'Judge the finds.', text: 'Everyone sees the same locked view. Your vote counts as ready; only the owner marks ready. Once everyone responds, the next finding opens automatically. More matches than rejections earns a point. Most points wins.', mood: 'winner' as const },
  ];
  let leaveDialog: HTMLDialogElement;
  let rulesDialog: HTMLDialogElement;
  let unsubscribe: (() => void) | undefined;
  let routeVersion = 0;
  let noticeTimer: ReturnType<typeof setTimeout>;
  const phases = ['Make a list', 'Hit the streets', 'Judge the finds'];
  let phaseIndex = $derived(!session || session.phase === 'lobby' ? -1 : session.phase === 'categories' ? 0 : ['searching', 'selection'].includes(session.phase) ? 1 : 2);
  let me = $derived(session?.players.find(p => p.id === session?.meId));
  let canReady = $derived(Boolean(session && connection === 'connected' && !(session.phase === 'categories' && !session.categories.length) && !(session.phase === 'review' && session.review?.finding.playerId !== session.meId)));
  let nextLabel = $derived(session?.phase === 'lobby' ? 'collect categories' : session?.phase === 'categories' ? 'start exploring' : session?.phase === 'selection' ? 'start the review' : session?.review && session.review.index + 1 === session.review.total ? 'see the final scores' : 'see the next finding');

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

<svelte:head><title>{session ? `${session.id} · ${session.phase === 'review' ? 'Judge the finds' : session.phase === 'searching' ? 'Hit the streets' : 'The spotters club'}` : 'Find the weird. Bring your friends.'} — Stadt Land Streetview</title></svelte:head>

<a class="skip-link" href="#main-content" onclick={event => { event.preventDefault(); document.getElementById('main-content')?.focus(); }}>Skip to content</a>
<div class="app-shell" class:in-game={Boolean(session)}>
<header class="site-header"><button class="brand" aria-label="Stadt Land Streetview home" onclick={() => session ? leaveDialog.showModal() : location.hash = ''}><GameLogo compact/></button>
  {#if session}<nav class="phase-nav" aria-label="Game progress">{#each phases as phase, i}<span class:current={phaseIndex === i} class:complete={phaseIndex > i}><span class="nav-number">{#if phaseIndex > i}<Icon name="check" size={12}/>{:else}{i + 1}{/if}</span>{phase}</span>{#if i < 2}<span class="nav-line"></span>{/if}{/each}</nav>{/if}
  <div class="header-actions">{#if session}<button class="room-code" title="Copy invite link" onclick={copyInvite}>ROOM <strong>{session.id}</strong><Icon name="copy" size={15}/></button><button class="icon-button leave-button" aria-label="Leave room" onclick={() => leaveDialog.showModal()}><Icon name="exit" size={20}/></button>{:else}{/if}<button class="icon-button rules-button" aria-label="How to play" onclick={() => { rulesPage = 0; showRules = true; rulesDialog.showModal(); }}><Icon name="info" size={21}/></button></div>
</header>

<div class="mode-banner" class:connection-warning={session && connection !== 'connected'}><span class="tiny-dot"></span>{#if session && connection !== 'connected'}{connection === 'connecting' ? 'Connecting to the crew…' : 'Reconnecting… Your place is saved. Actions resume when connected.'}{:else if session?.demo}SOLO DEMO <span>Illustrated scenes · Mika and Jules are simulated spotters · no Google Maps requests</span>{:else if session && mockMode}LOCAL PROTOTYPE <span>Rooms sync across tabs in this browser · connect Axum to play across devices</span>{:else}OFFICIAL BUSINESS <span>Find something ordinary. Make it everyone’s problem.</span>{/if}</div>
{#if error}<div class="alert" role="alert"><Icon name="info"/><span>{error}</span><button class="icon-button" aria-label="Dismiss error" onclick={() => error = ''}><Icon name="close" size={18}/></button></div>{/if}
{#if notice}<div class="toast" role="status"><Icon name="check" size={18}/>{notice}</div>{/if}

<div id="main-content" tabindex="-1">
{#if loading}<main class="loading-page"><span class="spinner"></span><h2>Finding your people…</h2></main>
{:else if !session}<Home {busy} {roomCode} oncreate={(name, duration, demo) => enter(name, duration, demo)} onjoin={(code, name) => enter(name, 600, false, code)}/>
{:else}{#key `${session.id}:${session.meId}`}<MapsSession><main class="game-main phase-{session.phase}">
  {#if session.phase === 'lobby'}<Lobby {session} oncopy={copyInvite}/>
  {:else if session.phase === 'categories'}<Categories {session} {busy} onaction={act}/>
  {:else if session.phase === 'searching'}<Search {session} {busy} onaction={act}/>
  {:else if session.phase === 'selection'}<Selection {session} {busy} onaction={act}/>
  {:else if session.phase === 'review' && session.review}<Review {session} {busy} onaction={act}/>
  {:else if session.phase === 'finished'}<Results {session} onhome={() => leaveDialog.showModal()}/>{/if}
  {#if !['searching', 'finished'].includes(session.phase)}<ReadyBar {session} {busy} disabled={!canReady} next={nextLabel} onready={() => session && act({ type: 'ready', stepId: session.stepId, ready: !me?.ready })}/>{/if}
</main></MapsSession>{/key}{/if}
</div>
</div>

<dialog bind:this={leaveDialog} class="modal"><div class="modal-body"><span class="round-icon"><Icon name="exit" size={25}/></span><h2>Taking a different route?</h2><p>Leaving removes you from the readiness requirement. Your submitted findings and earned points stay in the round.</p><div class="modal-actions"><button class="button secondary" onclick={() => leaveDialog.close()}>Stay with the group</button><button class="button primary" disabled={busy} onclick={leave}>Leave room <Icon name="arrow" size={17}/></button></div></div></dialog>
<dialog bind:this={rulesDialog} class="modal rules-modal" onclose={() => showRules = false}>{#if showRules}<div class="modal-body"><button class="icon-button modal-close" aria-label="Close game rules" onclick={() => rulesDialog.close()}><Icon name="close"/></button><span class="eyebrow">SPOTTER ORIENTATION</span><h2>How to play</h2><div class="rules-step"><div class="rules-character"><Spotter seed="orientation-officer" mood={rules[rulesPage].mood}/></div><div><span class="step-number">0{rulesPage + 1}</span><h3>{rules[rulesPage].title}</h3><p>{rules[rulesPage].text}</p></div></div><div class="rules-navigation"><span class="small">Ready is reversible until everyone’s ready.</span><Pager page={rulesPage} pages={4} label="rules" onpage={value => rulesPage = value}/></div><button class="button primary full" onclick={() => rulesDialog.close()}>Got it. Let’s spot. <Icon name="arrow"/></button></div>{/if}</dialog>
