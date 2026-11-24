<script lang="ts">
    import type { Action, SessionState } from "../lib/api/types";
    import Icon from "../lib/components/Icon.svelte";
    import Avatar from "../lib/components/Avatar.svelte";
    import Pager from "../lib/components/Pager.svelte";
    import StreetView from "../lib/maps/StreetView.svelte";
    export let session: SessionState;
    export let onaction: (action: Action) => Promise<boolean>;
    export let onskip: () => void;
    $: showcase = session.showcase!;
    $: owner = session.players.find(
        (p) => p.id === showcase.souvenir.playerId,
    )!;
    $: own = owner.id === session.meId;
</script>

<section class="phase-heading">
    <div>
        <span class="eyebrow">NO POINTS. JUST MEMORIES.</span>
        <h1>STREET <span>SOUVENIRS.</span></h1>
    </div>
    <button class="button secondary" onclick={onskip}>Skip the souvenirs</button
    >
</section>
<div class="review-layout">
    <section class="review-main panel">
        <div class="review-title">
            <div>
                <span class="eyebrow">LOOK WHAT TURNED UP</span>
                <h2 title={owner.name}>
                    {own ? "Your" : `${owner.name}’s`} souvenirs
                </h2>
            </div>
            <div class="finding-owner">
                <Avatar player={owner} small /><span
                    ><strong title={owner.name}
                        >{owner.name}{own ? " (you)" : ""}</strong
                    ><small>BROUGHT THESE BACK</small></span
                >
            </div>
        </div>
        <div class="view-stage">
            <div class="view-frame">
                {#key showcase.souvenir.view.demoScene !== undefined ? "demo" : "google"}<StreetView
                        view={showcase.souvenir.view}
                    />{/key}
            </div>
        </div>
        <div class="review-bottom">
            <span
                ><Icon name="camera" size={15} /> ORIGINAL FRAME · SAME PAGE FOR
                EVERYONE</span
            ><span
                >SPOTTER {showcase.playerIndex + 1} OF {showcase.playerTotal}</span
            >
        </div>
    </section>
    <aside class="panel voting-panel">
        <span class="eyebrow">SHOW AND TELL</span>
        <h2>{own ? "Go on, explain yourself." : "Well, look at that."}</h2>
        <p>
            {own
                ? "These are yours. Talk the crew through what you were thinking."
                : "Not on the list, still worth the detour. Sit back and enjoy."}
        </p>
        <div class="own-finding-character">
            <Avatar player={owner} large mood="searching" />
        </div>
        <div class="souvenir-pages">
            <span class="small"
                >Souvenir {showcase.index + 1} of {showcase.total}</span
            ><Pager
                page={showcase.index}
                pages={showcase.total}
                label="souvenirs"
                onpage={(index) =>
                    onaction({
                        type: "show-souvenir",
                        stepId: session.stepId,
                        index,
                    })}
            />
        </div>
        <p class="small muted">Anyone can turn the page for the whole crew.</p>
    </aside>
</div>

<style>
    .souvenir-pages {
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-wrap: wrap;
        gap: 8px;
    }
</style>
