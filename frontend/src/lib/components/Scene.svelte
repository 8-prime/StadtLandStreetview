<script lang="ts">
  export let scene = 0;
  export let decorative = false;
  const palettes = [
    { sky: '#dce8e2', wall: '#edcba7', roof: '#b16f56', second: '#b9c5ad', car: '#d8a34f', tree: '#718f73' },
    { sky: '#e8e6d9', wall: '#e2b9a8', roof: '#8f6557', second: '#c7c8b8', car: '#75999b', tree: '#8c9e6d' },
    { sky: '#dde6ed', wall: '#e5d8b6', roof: '#b17766', second: '#a9bdb8', car: '#bd7460', tree: '#73896d' },
  ];
  $: p = palettes[scene % palettes.length];
</script>

<svg class="scene" viewBox="0 0 1000 650" preserveAspectRatio="xMidYMid slice" role={decorative ? 'presentation' : 'img'} aria-label={decorative ? undefined : 'Illustrated demo street with houses, a parked car, and a person walking a dog'}>
  <rect width="1000" height="650" fill={p.sky}/>
  <circle cx="800" cy="110" r="55" fill="#f6edce" opacity=".9"/>
  <path d="M0 200Q130 140 270 200T550 190T1000 180V410H0" fill="#c5d1be"/>
  <path d="M0 410h1000v240H0" fill="#b1afa5"/>
  <path d="M0 458h1000M0 566h1000" stroke="#ece8db" stroke-width="6"/>
  <path d="M40 514h100m90 0h100m90 0h100m90 0h100m90 0h100" stroke="#f2ecdb" stroke-width="4"/>
  <path d="m0 390 1000-12v56H0" fill="#dfd8c7"/>
  <path d="M70 390V184l162-86 159 86v206" fill={p.wall}/>
  <path d="m48 191 184-104 182 104-14 20-168-96L61 211" fill={p.roof}/>
  <path d="M389 390V225h209v165" fill={p.second}/>
  <path d="M381 216h225v23H381" fill="#7c8171"/>
  <path d="M683 390V180h251v210" fill="#ece4cc"/>
  <path d="M670 175h275v22H670" fill="#a88871"/>
  {#each [120, 207, 294] as x}
    {#each [206, 288] as y}
      <rect {x} {y} width="43" height="57" rx="2" fill="#68817d"/>
      <path d={`M${x + 21} ${y}v57m-21-29h43`} stroke="#f3e2c9" stroke-width="4"/>
      <path d={`M${x - 5} ${y + 60}h53`} stroke="#b28e71" stroke-width="6"/>
    {/each}
  {/each}
  <path d="M215 390v-70a24 24 0 0 1 48 0v70" fill="#435f54"/>
  <circle cx="252" cy="353" r="3" fill="#e0b775"/>
  {#each [423, 508] as x}
    <rect {x} y="266" width="52" height="67" fill="#f5e9ce"/>
    <rect x={x + 7} y="273" width="38" height="53" fill="#69827b"/>
  {/each}
  <rect x="445" y="348" width="101" height="42" fill="#6e7b62"/>
  <path d="M713 330h191v60H713" fill="#8b9b82"/>
  <path d="M707 323h203l-15-25H722" fill="#c08464"/>
  {#each [728, 778, 828, 878] as x}<path d={`M${x} 299v24`} stroke="#ead5ae" stroke-width="22"/>{/each}
  <rect x="725" y="220" width="50" height="58" fill="#849b91"/>
  <rect x="839" y="220" width="50" height="58" fill="#849b91"/>
  <path d="M0 414h1000" stroke="#9c9b89" stroke-width="4"/>
  <ellipse cx="585" cy="444" rx="108" ry="12" fill="#7a7d70" opacity=".2"/>
  <path d="M484 414v-25l24-12 26-38h78l31 38 29 11v39H484" fill={p.car}/>
  <path d="m541 346-19 30h103l-21-30Z" fill="#536f6d"/>
  <path d="M570 345v32" stroke={p.car} stroke-width="6"/>
  <path d="M493 399h170" stroke="#fff1c7" stroke-width="3" opacity=".5"/>
  <circle cx="521" cy="423" r="19" fill="#47524c"/><circle cx="521" cy="423" r="9" fill="#dbd5c4"/>
  <circle cx="639" cy="423" r="19" fill="#47524c"/><circle cx="639" cy="423" r="9" fill="#dbd5c4"/>
  <path d="M75 412V277m-17 0h34" stroke="#68786c" stroke-width="7"/>
  <rect x="48" y="246" width="54" height="38" rx="5" fill="#7294a0"/><text x="68" y="274" fill="#f8f6e9" font-size="29" font-family="sans-serif">P</text>
  <path d="M955 412V292" stroke="#7d7960" stroke-width="16"/>
  <circle cx="951" cy="269" r="55" fill={p.tree}/><circle cx="919" cy="294" r="39" fill={p.tree}/><circle cx="982" cy="303" r="45" fill={p.tree}/>
  <path d="M34 415v-91" stroke="#7d7960" stroke-width="12"/><ellipse cx="30" cy="299" rx="52" ry="67" fill={p.tree}/>
  <ellipse cx="348" cy="419" rx="64" ry="8" fill="#697263" opacity=".17"/>
  <circle cx="325" cy="337" r="12" fill="#bd9472"/>
  <path d="M314 353h22l9 38h-37Z" fill="#3f695c"/>
  <path d="m318 391-7 26m20-26 11 26m-4-56 17 19" stroke="#46554b" stroke-width="8" stroke-linecap="round"/>
  <path d="m354 378 31 22" stroke="#5a6556" stroke-width="2"/>
  <path d="M377 401h28v13h-28Zm24-5 12-3 4 12h-17" fill="#b38358"/>
  <path d="M381 411v9m19-9v9m-23-15-8-8" stroke="#916943" stroke-width="4" stroke-linecap="round"/>
  <path d="M830 409v-44m-15 0h30m-19-45h8v45h-8" stroke="#586c60" stroke-width="4"/>
  <path d="M815 320h30l-5-24h-20Z" fill="#6c7c68"/>
  <path d="m124 598 22-21m-15 34 23-22m735-9 22-21m-15 35 24-23" stroke="#d2d0bf" stroke-width="3"/>
  <path d="M0 0h1000v650H0Z" fill="none" stroke="#234c3d" stroke-opacity=".06" stroke-width="16"/>
</svg>
