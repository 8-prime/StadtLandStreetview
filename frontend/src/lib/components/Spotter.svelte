<script lang="ts">
  import { makeSpotter, type SpotterMood } from './spotter';
  export let seed: string;
  export let mood: SpotterMood = 'idle';
  export let backdrop = true;
  $: person = makeSpotter(seed);
</script>

<svg class="spotter" class:spotter-offline={mood === 'offline'} viewBox="0 0 160 180" fill="none" aria-hidden="true" data-spotter={seed}>
  {#if backdrop}<path d="M15 91a65 65 0 1 1 130 0v58a18 18 0 0 1-18 18H33a18 18 0 0 1-18-18V91Z" fill={person.backdrop}/>{/if}
  <g stroke="#202b45" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round">
    <path d="M31 166v-21c0-23 19-37 49-37s49 14 49 37v21H31Z" fill={person.shirt}/>
    <path d="M65 108v15q15 16 30 0v-15" fill={person.skin}/>
    <path d="m47 132-5 34m71-34 5 34"/>
    <path d="m96 122-35 44" stroke="#202b45" stroke-width="9"/>
    <path d="m96 122-35 44" stroke="#fff4d7" stroke-width="4"/>
    <ellipse cx="46" cy="81" rx="9" ry="12" fill={person.skin}/>
    <ellipse cx="114" cy="81" rx="9" ry="12" fill={person.skin}/>
    <path d="M47 62c0-25 66-25 66 0v29c0 23-14 35-33 35S47 114 47 91V62Z" fill={person.skin}/>
    {#if person.hairStyle === 0}<path d="M47 77V57c0-27 66-31 66 0v17l-10-22q-25 14-47 6l-9 19Z" fill={person.hair}/>
    {:else if person.hairStyle === 1}<path d="m46 78-4-25 10-6-1-11 17 1 8-11 14 8 17-3 2 13 12 9-7 25-7-18q-26 8-52 0l-9 18Z" fill={person.hair}/>
    {:else}<path d="M46 88V61c-1-24 12-33 34-33s35 9 34 33v27l-11-19-2-18q-21 18-45 8l-1 18-9 11Z" fill={person.hair}/>{/if}
    {#if person.hat === 0}<path d="M44 53c0-27 72-27 72 0v7H44v-7Z" fill={person.shirt}/><path d="M77 38v17m-43 7h88q-8 11-45 4" fill={person.shirt}/>
    {:else if person.hat === 1}<path d="m45 60 8-27q27-10 54 0l8 27H45Z" fill="#fff4d7"/><path d="m40 61 5-10h70l5 10q-40 15-80 0Z" fill={person.shirt}/>
    {:else if person.hat === 2}<path d="M47 57V46a33 33 0 0 1 66 0v11H47Z" fill={person.shirt}/><path d="M44 50h72v14H44z" fill={person.shirt}/><path d="M56 54v5m12-5v5m12-5v5m12-5v5m12-5v5" stroke-width="2"/>{/if}
    {#if mood === 'winner'}<path d="m53 36-5-23 19 10L80 7l13 16 19-10-5 23H53Z" fill="#ffce43"/><circle cx="80" cy="26" r="3" fill="#fff4d7" stroke-width="2"/>{/if}
    <path d="m59 77 10-2m22 0 10 2" stroke-width="3"/>
    {#if mood === 'ready'}<path d="m62 87 4-3 4 3m20 0 4-3 4 3"/>
    {:else}<ellipse cx="66" cy="86" rx="2.5" ry="3.5" fill="#202b45" stroke="none"/><ellipse cx="94" cy="86" rx="2.5" ry="3.5" fill="#202b45" stroke="none"/>{/if}
    <path d="m79 87-3 10h6" stroke-width="2.5"/>
    {#if person.smile === 0 || mood === 'ready' || mood === 'winner'}<path d="M69 105q11 12 22 0" fill="#fff4d7" stroke-width="2.5"/>
    {:else if person.smile === 1}<path d="M72 108q8 4 16-2" stroke-width="2.5"/>
    {:else}<path d="M73 107h14" stroke-width="2.5"/>{/if}
    {#if person.glasses === 1}<circle cx="65" cy="86" r="11" stroke-width="3"/><circle cx="95" cy="86" r="11" stroke-width="3"/><path d="M76 84h8m-30-1-7-2m59 2 7-2" stroke-width="3"/>
    {:else if person.glasses === 2}<path d="M53 80h23v14H53zm31 0h23v14H84z" fill="#3b5472"/><path d="M76 84h8M58 84l5 6m27-6 5 6" stroke="#fff4d7" stroke-width="2"/>{/if}
    {#if person.accessory === 0}<rect x="73" y="139" width="31" height="23" rx="4" fill="#fff4d7"/><path d="M80 139v-5h10v5"/><circle cx="89" cy="151" r="7" fill="#78adc6"/>
    {:else if person.accessory === 1}<path d="M82 135h24v31H82z" fill="#fff4d7"/><path d="M88 143h12m-12 6h12m-12 6h7" stroke-width="2"/>
    {:else}<path d="m79 137-3 20h12l2-20m6 0-2 20h12l2-20" fill="#4d6480"/><path d="M87 143h9"/>{/if}
  </g>
  {#if mood === 'searching'}<g stroke="#202b45" stroke-width="3.5" stroke-linejoin="round"><path d="m53 77 7-9h14l6 9 6-9h14l7 9v22H53V77Z" fill="#5c7794"/><path d="M74 84h12"/><circle cx="64" cy="89" r="15" fill="#202b45"/><circle cx="96" cy="89" r="15" fill="#202b45"/><circle cx="64" cy="89" r="9" fill="#a7d5dd" stroke="none"/><circle cx="96" cy="89" r="9" fill="#a7d5dd" stroke="none"/><path d="m60 86 5-3m27 3 5-3" stroke="#fff" stroke-width="3"/></g>{/if}
  {#if mood === 'ready'}<g stroke="#202b45" stroke-width="3"><circle cx="132" cy="145" r="17" fill="#b7df93"/><path d="m124 145 5 5 10-11" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/></g>{/if}
</svg>
