<script lang="ts">
    import { onMount } from "svelte";
    import type {
        Action,
        ProximityBand,
        SessionState,
        Viewpoint,
    } from "../lib/api/types";
    import { api } from "../lib/api";
    import ProximityRadio from "../lib/components/ProximityRadio.svelte";
    import Icon from "../lib/components/Icon.svelte";
    import { countryFlag, countryName } from "../lib/api/countries";
    import { formatClock } from "../lib/api/duration";
    import Pager from "../lib/components/Pager.svelte";
    import CategoryRail from "../lib/components/CategoryRail.svelte";
    import StreetView from "../lib/maps/StreetView.svelte";
    export let session: SessionState;
    export let busy = false;
    export let connected = true;
    export let proximity: ProximityBand = "inactive";
    // The banned country containing the last reported position, if any.
    export let banned: string | null = null;
    export let onaction: (action: Action) => Promise<boolean>;
    let categoryId = session.categories[0]?.id ?? "";
    let current: Viewpoint | null = null;
    let now = Date.now();
    let savedMessage = "";
    let savedPage = 0;
    // What the strip below the map lists.
    let showSouvenirs = false;
    let messageTimer: ReturnType<typeof setTimeout>;
    const size = 3;
    const offset = session.serverTime - Date.now();
    let mounted = false;
    let lastPosition = "";
    let lastSent = 0;
    $: if (!connected) {
        lastPosition = "";
        lastSent = 0;
    }
    $: remaining = Math.max(
        0,
        Math.ceil(((session.searchEndsAt ?? 0) - (now + offset)) / 1000),
    );
    $: timer = formatClock(remaining);
    $: reportsPosition =
        session.settings.proximityEnabled ||
        session.settings.bannedCountries.length > 0;
    $: offLimits =
        current && banned
            ? `${countryFlag(banned)} ${countryName(banned)} is off-limits this round. Keep moving.`
            : "";
    $: category = session.categories.find((c) => c.id === categoryId);
    $: candidates = session.ownFindings.filter(
        (f) => f.categoryId === categoryId,
    );
    $: souvenirs = session.ownSouvenirs;
    $: saved = showSouvenirs ? souvenirs : candidates;
    $: pages = Math.max(1, Math.ceil(saved.length / size));
    $: if (savedPage >= pages) savedPage = pages - 1;
    function reportPosition(force = false) {
        if (!mounted || !connected || !reportsPosition) return;
        const report = {
            stepId: session.stepId,
            position: current?.position ?? null,
        };
        const key = JSON.stringify(report);
        if (
            force ||
            key !== lastPosition ||
            (report.position && Date.now() - lastSent >= 5000)
        ) {
            api.reportPosition(session.id, report);
            lastPosition = key;
            lastSent = Date.now();
        }
    }
    function viewChanged(value: Viewpoint | null) {
        const wasInStreet = Boolean(current);
        current = value;
        if (!value && wasInStreet) reportPosition(true);
    }
    onMount(() => {
        mounted = true;
        const interval = setInterval(() => (now = Date.now()), 250);
        const presence = setInterval(() => reportPosition(), 1000);
        reportPosition();
        return () => {
            mounted = false;
            clearInterval(interval);
            clearInterval(presence);
            clearTimeout(messageTimer);
            if (reportsPosition)
                api.reportPosition(session.id, {
                    stepId: session.stepId,
                    position: null,
                });
        };
    });
    function announce(message: string) {
        savedMessage = message;
        clearTimeout(messageTimer);
        messageTimer = setTimeout(() => (savedMessage = ""), 3500);
    }
    async function saveSouvenir() {
        if (!current) return;
        const nextPage = Math.floor(souvenirs.length / size);
        if (await onaction({ type: "save-souvenir", view: current })) {
            showSouvenirs = true;
            savedPage = nextPage;
            announce("Souvenir packed. Show it off after the scores.");
        }
    }
    async function save() {
        if (!current || !category || offLimits) return;
        const nextPage = Math.floor(candidates.length / size);
        if (
            await onaction({ type: "save-finding", categoryId, view: current })
        ) {
            showSouvenirs = false;
            savedPage = nextPage;
            announce("Evidence secured. Nice spotting.");
        }
    }
</script>

<section class="phase-heading">
    <div>
        <span class="eyebrow">02 / HIT THE STREETS</span>
        <h1>EYES ON <span>EVERYTHING.</span></h1>
    </div>
    <div class="search-instruments">
        {#if session.settings.proximityEnabled}<ProximityRadio
                band={proximity}
                {connected}
                active={Boolean(current)}
            />{/if}
        <div class="search-clock" class:urgent={remaining <= 30}>
            <Icon name="clock" size={23} /><strong
                aria-label={`${remaining} seconds remaining`}>{timer}</strong
            ><span>{remaining ? "LEFT TO SPOT" : "SYNCING…"}</span>
        </div>
    </div>
</section>
<div class="search-workspace">
    <CategoryRail
        {session}
        selected={categoryId}
        onselect={(id) => {
            categoryId = id;
            showSouvenirs = false;
            savedPage = 0;
            savedMessage = "";
        }}
    />
    <section class="map-workspace panel">
        <div class="map-topline">
            <span><Icon name="compass" size={16} /> FIELD OF OPERATIONS</span
            ><span>NO SEARCH BAR. NO EXCUSES.</span>
        </div>
        <div class="view-stage">
            <div class="view-frame">
                <StreetView
                    explore
                    demo={session.demo}
                    start={session.start}
                    onview={viewChanged}
                />
            </div>
        </div>
        <div class="save-view-bar">
            <div>
                <span class="eyebrow">CURRENT ASSIGNMENT</span><strong
                    title={category?.text}>{category?.text}</strong
                ><span
                    class="small muted"
                    class:off-limits={offLimits && !savedMessage}
                    role="status"
                    >{savedMessage ||
                        offLimits ||
                        (current
                            ? "Frame it. Claim it."
                            : "Drop Pegman onto a street to start spotting.")}</span
                >
            </div>
            <button
                class="button secondary"
                title="Keep something fun that fits no category"
                onclick={saveSouvenir}
                disabled={busy ||
                    !current ||
                    remaining === 0 ||
                    souvenirs.length >= 12}
                ><Icon name="spark" /> Keep souvenir {#if souvenirs.length}<span
                        class="button-count">{souvenirs.length}</span
                    >{/if}</button
            ><button
                class="button primary"
                onclick={save}
                disabled={busy ||
                    !current ||
                    Boolean(offLimits) ||
                    remaining === 0 ||
                    candidates.length >= 12}
                ><Icon name="camera" /> Save view {#if candidates.length}<span
                        class="button-count">{candidates.length}</span
                    >{/if}</button
            >
        </div>
    </section>
</div>
<section class="saved-strip">
    <div class="saved-strip-label">
        <Icon name={showSouvenirs ? "spark" : "camera"} size={19} />
        {#each [false, true] as souvenirTab}<button
                class="strip-tab"
                aria-pressed={showSouvenirs === souvenirTab}
                onclick={() => {
                    showSouvenirs = souvenirTab;
                    savedPage = 0;
                }}>{souvenirTab ? "Souvenirs" : "Your evidence"}</button
            >{/each}<span class="small">PRIVATE</span>
    </div>
    <div class="saved-chips">
        {#each saved.slice(savedPage * size, (savedPage + 1) * size) as item, i}<span
                class="saved-chip"
                ><Icon name={showSouvenirs ? "spark" : "check"} size={14} />
                {showSouvenirs ? "Souvenir" : "View"}
                {savedPage * size + i + 1}<button
                    class="icon-button"
                    disabled={busy}
                    aria-label={`Remove ${showSouvenirs ? "souvenir" : "saved view"} ${savedPage * size + i + 1}`}
                    onclick={() =>
                        onaction(
                            showSouvenirs
                                ? {
                                      type: "delete-souvenir",
                                      souvenirId: item.id,
                                  }
                                : {
                                      type: "delete-finding",
                                      findingId: item.id,
                                  },
                        )}><Icon name="close" size={13} /></button
                ></span
            >{:else}<span class="muted small"
                >{showSouvenirs
                    ? "Fun things that fit no category go here."
                    : "Your next great find goes here."}</span
            >{/each}
    </div>
    <Pager
        page={savedPage}
        {pages}
        label={showSouvenirs ? "souvenirs" : "saved views"}
        onpage={(value) => (savedPage = value)}
    />
</section>

<style>
    .off-limits {
        color: #b43c27;
        font-weight: 800;
    }
    .strip-tab {
        margin-left: 4px;
        padding: 2px 0;
        border: 0;
        border-bottom: 3px solid transparent;
        background: none;
        color: var(--muted);
        font-family: var(--heading);
        font-size: 15px;
        font-weight: 800;
        white-space: nowrap;
        cursor: pointer;
    }
    .strip-tab[aria-pressed="true"] {
        border-color: var(--ink);
        color: var(--ink);
    }
    @media (max-width: 900px) {
        .strip-tab {
            font-size: 12px;
        }
    }
    .search-instruments {
        display: flex;
        align-items: center;
        gap: 18px;
    }
    @media (max-height: 780px) {
        .search-instruments {
            gap: 12px;
        }
    }
</style>
