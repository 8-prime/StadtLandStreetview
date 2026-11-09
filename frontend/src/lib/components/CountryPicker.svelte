<script lang="ts">
    import Icon from "./Icon.svelte";
    import Pager from "./Pager.svelte";
    import {
        countryFlag,
        countryName,
        searchCountries,
    } from "../api/countries";

    let {
        selected = $bindable([]),
        disabled = false,
        onchange,
    }: {
        selected?: string[];
        disabled?: boolean;
        // Called when the dialog closes with a changed list; without it, `selected` updates directly.
        onchange?: (codes: string[]) => void;
    } = $props();
    let dialog: HTMLDialogElement;
    let search: HTMLInputElement;
    let query = $state("");
    let page = $state(0);
    // Edits stay local until the dialog closes, so a shared room gets one update per visit.
    let draft = $state<string[]>([]);
    const resultLimit = 8;
    const chipsPerPage = 8;
    const byName = (a: string, b: string) =>
        countryName(a).localeCompare(countryName(b), "en");
    let results = $derived(
        query.trim() ? searchCountries(query, draft).slice(0, resultLimit) : [],
    );
    let pages = $derived(Math.max(1, Math.ceil(draft.length / chipsPerPage)));
    let sorted = $derived([...selected].sort(byName));
    let summary = $derived(
        sorted.length
            ? sorted.slice(0, 5).map(countryFlag).join(" ") +
                  (sorted.length > 5 ? ` +${sorted.length - 5}` : "")
            : "None",
    );
    $effect(() => {
        if (page >= pages) page = pages - 1;
    });

    function open() {
        draft = [...sorted];
        query = "";
        page = 0;
        dialog.showModal();
        search.focus();
    }
    function close() {
        const key = (codes: string[]) => [...codes].sort().join();
        if (key(draft) === key(selected)) return;
        // A controlled picker shows the new list once the room confirms it.
        if (onchange) onchange(draft);
        else selected = draft;
    }
    function add(code: string) {
        draft = [...draft, code].sort(byName);
        query = "";
        page = Math.floor(draft.indexOf(code) / chipsPerPage);
        search.focus();
    }
    function remove(code: string) {
        draft = draft.filter((value) => value !== code);
    }
</script>

<button
    type="button"
    class="room-option ban-option"
    {disabled}
    onclick={open}
    aria-label={`Off-limits countries: ${sorted.length ? sorted.map(countryName).join(", ") : "none"}`}
>
    <span
        ><strong>Off-limits countries</strong><small
            >No starts or evidence there.</small
        ></span
    >
    <span class="ban-summary" aria-hidden="true">{summary}</span>
</button>

<dialog
    bind:this={dialog}
    class="modal country-modal"
    aria-labelledby="country-modal-title"
    onclose={close}
>
    <div class="modal-body">
        <button
            type="button"
            class="icon-button modal-close"
            aria-label="Close off-limits countries"
            onclick={() => dialog.close()}><Icon name="close" /></button
        >
        <span class="eyebrow">ROOM RULES</span>
        <h2 id="country-modal-title">Off-limits countries</h2>
        <p>
            Nobody starts there, and evidence found there can’t be saved. Street
            View still works, so players can pass through.
        </p>
        <label for="country-search">Add a country</label>
        <input
            id="country-search"
            bind:this={search}
            bind:value={query}
            placeholder="Type a country or code, e.g. Germany or DE"
            autocomplete="off"
            spellcheck="false"
            onkeydown={(event) => {
                if (event.key === "Enter") {
                    event.preventDefault();
                    if (results[0]) add(results[0].code);
                }
            }}
        />
        <div class="country-results" aria-label="Matching countries">
            {#each results as country (country.code)}
                <button
                    type="button"
                    class="country-result"
                    onclick={() => add(country.code)}
                    ><span aria-hidden="true">{country.flag}</span><span
                        class="country-name">{country.name}</span
                    ><Icon name="plus" size={14} /></button
                >
            {:else}
                <span class="small muted"
                    >{query.trim()
                        ? "No matching country."
                        : "Start typing to find a country. Enter adds the first match."}</span
                >
            {/each}
        </div>
        <div class="country-selected">
            <span class="eyebrow"
                >{draft.length
                    ? `${draft.length} OFF-LIMITS`
                    : "THE WHOLE WORLD IS OPEN"}</span
            >
            <div class="country-chips">
                {#each draft.slice(page * chipsPerPage, (page + 1) * chipsPerPage) as code (code)}
                    <span class="saved-chip"
                        ><span aria-hidden="true">{countryFlag(code)}</span
                        >{countryName(code)}<button
                            type="button"
                            class="icon-button"
                            aria-label={`Allow ${countryName(code)} again`}
                            onclick={() => remove(code)}
                            ><Icon name="close" size={13} /></button
                        ></span
                    >
                {/each}
            </div>
            <div class="country-footer">
                <Pager
                    {page}
                    {pages}
                    label="off-limits countries"
                    onpage={(value) => (page = value)}
                />
                <span class="country-actions">
                    {#if draft.length}<button
                            type="button"
                            class="text-button"
                            onclick={() => {
                                draft = [];
                                page = 0;
                            }}>Clear all</button
                        >{/if}
                    <button
                        type="button"
                        class="button primary"
                        onclick={() => dialog.close()}
                        >Done <Icon name="check" size={16} /></button
                    >
                </span>
            </div>
        </div>
    </div>
</dialog>

<style>
    .ban-option {
        display: flex;
        align-items: center;
        gap: 9px;
        width: 100%;
        margin: 8px 0 0;
        padding: 8px;
        border: 1.5px dashed #202b4555;
        border-radius: 5px;
        color: var(--ink);
        background: transparent;
        font: inherit;
        text-align: left;
        cursor: pointer;
    }
    .ban-option:hover:not(:disabled) {
        border-color: var(--ink);
    }
    .ban-option:disabled {
        cursor: default;
        opacity: 0.6;
    }
    .ban-option > span:first-child {
        flex: 1;
        min-width: 0;
    }
    .ban-option strong {
        display: block;
        font-size: 11px;
    }
    .ban-option small {
        display: block;
        margin-top: 2px;
        font-size: 9px;
        font-weight: 600;
        color: var(--muted);
    }
    .ban-summary {
        max-width: 50%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        font-size: 12px;
        font-weight: 850;
    }
    @media (max-height: 780px) {
        .ban-option {
            margin-top: 6px;
            padding: 6px;
        }
    }
    .country-modal {
        width: 560px;
    }
    .country-modal .modal-body > h2 {
        margin-bottom: 8px;
    }
    .country-modal .modal-body > p {
        margin-bottom: 14px;
        line-height: 1.5;
    }
    .country-modal label {
        display: block;
        margin-bottom: 6px;
        font-size: 12px;
        font-weight: 850;
    }
    .country-results {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 6px;
        align-content: start;
        height: 140px;
        margin-top: 10px;
    }
    .country-result {
        display: flex;
        align-items: center;
        gap: 7px;
        min-width: 0;
        padding: 6px 9px;
        border: 1.5px solid var(--ink);
        border-radius: 5px;
        background: #fff;
        font-size: 12px;
        font-weight: 750;
        text-align: left;
    }
    .country-result:hover,
    .country-result:focus-visible {
        background: var(--yellow);
    }
    .country-name {
        flex: 1;
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .country-results > .small {
        grid-column: 1 / -1;
        padding: 8px 0;
    }
    .country-selected {
        margin-top: 12px;
        padding-top: 12px;
        border-top: 2px dashed #202b4530;
    }
    .country-chips {
        display: flex;
        flex-wrap: wrap;
        align-content: start;
        gap: 6px;
        height: 64px;
        margin: 8px 0 10px;
        overflow: hidden;
    }
    .country-footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
    }
    .country-actions {
        display: flex;
        align-items: center;
        gap: 14px;
    }
</style>
