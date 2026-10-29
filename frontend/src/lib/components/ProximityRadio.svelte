<script lang="ts">
  import type { ProximityBand } from '../api/types';
  export let band: ProximityBand = 'inactive';
  export let connected = true;
  export let active = false;
  const messages = { inactive: 'Pick a street', quiet: 'All quiet out there', nearby: 'Someone nearby', close: 'Very close!' };
</script>

<div class="field-radio" class:receiving={connected && (band === 'nearby' || band === 'close')} class:close={connected && band === 'close'} data-band={connected ? band : 'inactive'} role="status" aria-label="Neighbourhood radio" aria-live="polite" aria-atomic="true" title="Anonymous spotter signals: nearby within 5 km, very close within 250 m. No names or direction.">
  <span class="radio-antenna" aria-hidden="true"></span>
  <span class="radio-speaker" aria-hidden="true"><i></i><i></i><i></i><i></i></span>
  <div class="radio-screen"><span class="radio-name">Neighbourhood radio <b>5 km</b></span><strong>{!connected ? 'Reconnecting…' : band === 'inactive' && active ? 'Tuning in…' : messages[band]}</strong></div>
  <span class="radio-side" aria-hidden="true"><span class="radio-light"></span><span class="radio-dial"></span></span>
</div>

<style>
  .field-radio { position: relative; display: flex; align-items: center; gap: 8px; width: 238px; height: 50px; margin-top: 5px; padding: 6px 8px; flex-shrink: 0; background: var(--ink); border: 2px solid var(--ink); border-radius: 7px 7px 9px 7px; box-shadow: 3px 3px 0 var(--ink); }
  .radio-antenna { position: absolute; left: 14px; top: -8px; width: 6px; height: 8px; background: var(--ink); border-radius: 2px 2px 0 0; }
  .radio-speaker { display: flex; flex-direction: column; gap: 3px; width: 15px; flex-shrink: 0; }
  .radio-speaker i { height: 2px; width: 100%; border-radius: 2px; background: var(--paper); }
  .radio-screen { flex: 1; min-width: 0; padding: 3px 7px; border: 1.5px solid var(--ink); border-radius: 3px; background: #e3ebda; color: var(--ink); }
  .radio-name { display: flex; justify-content: space-between; align-items: center; gap: 7px; font-size: 7px; font-weight: 800; line-height: 1.1; white-space: nowrap; }
  .radio-name b { font-size: 7px; }
  .radio-screen strong { display: block; margin-top: 3px; font: 800 14px/1 var(--heading); white-space: nowrap; }
  .radio-side { display: flex; flex-direction: column; align-items: center; justify-content: space-between; height: 29px; }
  .radio-light { width: 6px; height: 6px; border: 1.5px solid #fff9e966; border-radius: 50%; background: #667189; }
  .radio-dial { width: 13px; height: 13px; border: 2px solid var(--paper); border-radius: 50%; background: var(--yellow); position: relative; }
  .radio-dial::after { content: ''; position: absolute; width: 2px; height: 4px; top: 0; left: 3.5px; background: var(--ink); transform: rotate(25deg); }
  .receiving { background: var(--yellow); }
  .receiving .radio-speaker i { background: var(--ink); }
  .receiving .radio-screen { background: var(--paper); }
  .receiving .radio-light { background: var(--mint); border-color: var(--ink); }
  .receiving .radio-dial { border-color: var(--ink); background: var(--paper); }
  .close { background: var(--coral); }
  @media (max-height: 780px) { .field-radio { width: 226px; height: 43px; padding: 4px 7px; } .radio-screen strong { font-size: 13px; } }
  @media (max-width: 1100px) { .field-radio { width: 207px; } .radio-screen strong { font-size: 12px; } }
</style>
