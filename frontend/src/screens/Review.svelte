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

<section class="phase-heading"><div><span class="eyebrow">03 / JUDGE THE FINDS</span><h1>MATCH OR <span>MASSIVE STRETCH?</span></h1></div><span class="phase-note"><Icon name="eye"/> Finding {review.index + 1} of {review.total}</span></section>
<div class="review-layout">
  <section class="review-main panel"><div class="review-title"><div><span class="eyebrow">THE ASSIGNMENT</span><h2 title={category.text}>{category.text}</h2></div><div class="finding-owner"><Avatar player={owner} small/><span><strong title={owner.name}>{owner.name}{own ? ' (you)' : ''}</strong><small>SUBMITTED THIS EVIDENCE</small></span></div></div><div class="view-stage"><div class="view-frame">{#key review.id}<StreetView view={review.finding.view}/>{/key}</div></div><div class="review-bottom"><span><Icon name="camera" size={15}/> ORIGINAL FRAME · LOCKED FOR EVERYONE</span><span>EXHIBIT {String(review.index + 1).padStart(2, '0')}</span></div></section>
  <aside class="panel voting-panel" class:result-accepted={review.result?.accepted}>
    {#if review.revealed && review.result}
      <span class="verdict-icon"><Icon name={review.result.accepted ? 'check' : 'close'} size={34}/></span><span class="eyebrow">THE GROUP HAS SPOKEN</span><h2>{review.result.accepted ? 'That’s a match.' : 'Not quite a match.'}</h2><p>{review.result.accepted ? `One point for ${owner.name}. Good eye, spotter.` : 'Interesting evidence. Unconvinced jury.'}</p>
      <div class="vote-totals"><div><strong>{review.result.accepts}</strong><span>Matches</span></div><div><strong>{review.result.rejects}</strong><span>Doesn’t match</span></div></div>
      <p class="result-note">{review.result.accepts === review.result.rejects ? 'A tied vote doesn’t count as a match.' : 'Majority verdict. Case closed.'}</p><div class="soft-note"><Icon name="arrow" size={18}/><p>Ready? {review.index + 1 === review.total ? 'The final scores are next.' : 'On to the next piece of evidence.'}</p></div>
    {:else}
      <span class="eyebrow">YOUR VERDICT</span><h2>{own ? 'A moment for your find.' : 'A match, or a stretch?'}</h2><p>{own ? 'Your evidence. Their verdict. Sit this vote out, then mark ready to reveal.' : 'Does the saved view fit the assignment? Cast your vote, then mark ready.'}</p>
      {#if !own}<div class="vote-options"><button class:chosen={review.ownVote === 'accept'} class="vote-option accept" disabled={busy} aria-pressed={review.ownVote === 'accept'} onclick={() => onaction({ type: 'vote', itemId: review.id, vote: 'accept' })}><Icon name="check" size={25}/><span>Matches</span>{#if review.ownVote === 'accept'}<span class="vote-selected">YOUR PICK</span>{/if}</button><button class:chosen={review.ownVote === 'reject'} class="vote-option reject" disabled={busy} aria-pressed={review.ownVote === 'reject'} onclick={() => onaction({ type: 'vote', itemId: review.id, vote: 'reject' })}><Icon name="close" size={25}/><span>Doesn’t match</span>{#if review.ownVote === 'reject'}<span class="vote-selected">YOUR PICK</span>{/if}</button></div>
      {:else}<div class="own-finding-character"><Avatar player={owner} large mood="searching"/><span>NO SELF-ENDORSEMENTS.</span></div>{/if}
      <div class="vote-progress"><span><strong>{review.votesCast}</strong> of {review.eligibleVoters} votes in</span><div class="progress-track"><span style={`width:${review.eligibleVoters ? review.votesCast / review.eligibleVoters * 100 : 0}%`}></span></div></div><p class="small muted">{own ? 'You can’t vote on your own finding.' : 'Votes stay private. Change yours until voting closes.'}</p>
    {/if}
  </aside>
</div>
