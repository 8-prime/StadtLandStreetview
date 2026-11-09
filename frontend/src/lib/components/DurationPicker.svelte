<script lang="ts">
    import {
        clampDuration,
        durationPresets,
        maxDuration,
    } from "../api/duration";

    let {
        value = $bindable(600),
        id,
        disabled = false,
        onchange,
    }: {
        value?: number;
        id: string;
        disabled?: boolean;
        // Called only with a valid duration that differs from the current value.
        onchange?: (seconds: number) => void;
    } = $props();

    let customRequested = $state(false);
    let hoursInput: HTMLInputElement | undefined = $state();
    let minutesInput: HTMLInputElement | undefined = $state();
    let custom = $derived(
        customRequested ||
            !durationPresets.some((preset) => preset.seconds === value),
    );
    let hours = $derived(Math.floor(value / 3600));
    let minutes = $derived(Math.floor((value % 3600) / 60));

    function commit(seconds: number) {
        if (seconds === value) return;
        value = seconds;
        onchange?.(seconds);
    }
    function choose(event: Event) {
        const choice = (event.currentTarget as HTMLSelectElement).value;
        customRequested = choice === "custom";
        if (!customRequested) commit(Number(choice));
    }
    function commitOnEnter(event: KeyboardEvent) {
        if (event.key === "Enter") commitCustom();
    }
    function commitCustom() {
        if (!hoursInput || !minutesInput) return;
        const seconds = clampDuration(
            (Number(hoursInput?.value) || 0) * 3600 +
                (Number(minutesInput?.value) || 0) * 60,
        );
        // Show the clamped value even when it matches the current duration.
        if (hoursInput) hoursInput.value = String(Math.floor(seconds / 3600));
        if (minutesInput)
            minutesInput.value = String(Math.floor((seconds % 3600) / 60));
        commit(seconds);
    }
</script>

<!-- Hours and minutes commit together, once focus leaves the picker or on Enter. -->
<div
    class="duration-picker"
    class:custom
    onfocusout={(event) => {
        if (
            custom &&
            !(event.currentTarget as HTMLElement).contains(
                event.relatedTarget as Node | null,
            )
        )
            commitCustom();
    }}
>
    <select
        {id}
        {disabled}
        value={custom ? "custom" : String(value)}
        onchange={choose}
    >
        {#each durationPresets as preset (preset.seconds)}<option
                value={String(preset.seconds)}>{preset.label}</option
            >{/each}
        <option value="custom">{custom ? "Custom" : "Custom time…"}</option>
    </select>
    {#if custom}
        <span class="duration-field"
            ><input
                bind:this={hoursInput}
                type="number"
                inputmode="numeric"
                min="0"
                max={maxDuration / 3600}
                step="1"
                value={hours}
                {disabled}
                aria-label="Hours on the streets"
                onkeydown={commitOnEnter}
            /><span aria-hidden="true">h</span></span
        >
        <span class="duration-field"
            ><input
                bind:this={minutesInput}
                type="number"
                inputmode="numeric"
                min="0"
                max="59"
                step="1"
                value={minutes}
                {disabled}
                aria-label="Minutes on the streets"
                onkeydown={commitOnEnter}
            /><span aria-hidden="true">min</span></span
        >
    {/if}
</div>

<style>
    .duration-picker {
        display: flex;
        gap: 6px;
        min-width: 0;
    }
    .duration-picker select {
        flex: 1;
        min-width: 0;
    }
    .duration-picker.custom select {
        flex: 0 0 92px;
    }
    .duration-field {
        position: relative;
        flex: 1 1 0;
        min-width: 0;
    }
    .duration-field input {
        padding: 0 32px 0 6px;
        text-align: right;
        -moz-appearance: textfield;
        appearance: textfield;
    }
    .duration-field input::-webkit-inner-spin-button,
    .duration-field input::-webkit-outer-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }
    .duration-field > span {
        position: absolute;
        right: 10px;
        top: 50%;
        transform: translateY(-50%);
        font-size: 11px;
        font-weight: 850;
        color: var(--muted);
        pointer-events: none;
    }
</style>
