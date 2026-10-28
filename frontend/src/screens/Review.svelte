<script lang="ts">
  import type { Action, SessionState } from '../lib/api/types';
  import Icon from '../lib/components/Icon.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import StreetView from '../lib/maps/StreetView.svelte';
  export let session: SessionState;
  export let busy = false;
  export let onaction: (action: Action) => Promise<boolean>;
  $: review = session.review!;
  $: owner = session.players.find(p => p.id === review.finding.playerId)!;
  $: category = session.categories.find(c => c.id === review.finding.categoryId)!;
  $: own = review.finding.playerId === session.meId;
</script>

<section class="phase-heading compact"><div><span class="eyebrow">03 / MAKE THE CALL</span><h1>Does it <em>make the match?</em></h1><p>One finding at a time. The same view for everyone.</p></div><span class="phase-note"><Icon name="eye"/> Finding {review.index + 1} of {review.total}</span></section>
<div class="review-layout"><section class="review-main"><div class="review-title"><div><span class="eyebrow">LOOKING FOR</span><h2>{category.text}</h2></div><div class="finding-owner"><Avatar player={owner} small/><span><strong>{owner.name}{own ? ' (you)' : ''}</strong><small>’s discovery</small></span></div></div><div class="review-viewport">{#key review.id}<StreetView view={review.finding.view}/>{/key}<span class="frame-tag"><Icon name="camera" size={14}/> ORIGINAL FRAMING · VIEW LOCKED</span></div><div class="review-bottom"><span><Icon name="users" size={17}/> Everyone is viewing this finding.</span><span>{review.index + 1} / {review.total}</span></div></section>
  <aside class="panel voting-panel">
    {#if review.revealed && review.result}<span class="round-icon" class:accepted={review.result.accepted}><Icon name={review.result.accepted ? 'check' : 'close'} size={30}/></span><span class="eyebrow">THE GROUP HAS SPOKEN</span><h2>{review.result.accepted ? 'That’s a match.' : 'Not quite a match.'}</h2><p>{review.result.accepted ? `One point for ${owner.name}. A good eye goes a long way.` : 'A worthy detour, even if it didn’t make the cut.'}</p><div class="vote-totals"><div><strong>{review.result.accepts}</strong><span>Matches</span></div><div><strong>{review.result.rejects}</strong><span>Doesn’t match</span></div></div>{#if review.result.accepts === review.result.rejects}<p class="small">Tied votes don’t count as a match.</p>{/if}<div class="soft-note"><Icon name="info" size={18}/><p>Mark yourself ready once you’ve seen the result. {review.index + 1 === review.total ? 'Next up: the final scores.' : 'Then we’ll move on together.'}</p></div>
    {:else}<span class="round-icon"><Icon name="eye" size={28}/></span><span class="eyebrow">YOUR VERDICT</span><h2>{own ? 'A moment for your find.' : 'A match, or a stretch?'}</h2><p>{own ? 'This one is yours, so you sit out the vote. Mark yourself ready when you’re happy to reveal.' : 'Judge the saved view against the category. Your vote stays private until everyone is ready.'}</p>{#if !own}<div class="vote-options"><button class:chosen={review.ownVote === 'accept'} class="vote-option accept" disabled={busy} aria-pressed={review.ownVote === 'accept'} onclick={() => onaction({ type: 'vote', itemId: review.id, vote: 'accept' })}><Icon name="check"/><span>Matches</span>{#if review.ownVote === 'accept'}<span class="vote-selected">Selected</span>{/if}</button><button class:chosen={review.ownVote === 'reject'} class="vote-option reject" disabled={busy} aria-pressed={review.ownVote === 'reject'} onclick={() => onaction({ type: 'vote', itemId: review.id, vote: 'reject' })}><Icon name="close"/><span>Doesn’t match</span>{#if review.ownVote === 'reject'}<span class="vote-selected">Selected</span>{/if}</button></div>{/if}<div class="vote-progress"><span><strong>{review.votesCast}</strong> of {review.eligibleVoters} votes in</span><div class="progress-track"><span style={`width:${review.eligibleVoters ? review.votesCast / review.eligibleVoters * 100 : 0}%`}></span></div></div><p class="small muted">{own ? 'You can’t vote on your own finding.' : 'You can change your vote until voting closes.'}</p>{/if}
  </aside>
</div>
