<script lang="ts">
  const roads = [
    'M 180 -100 C 410 20 380 170 175 195 S -65 350 90 455 S 230 625 65 690 S -20 905 245 825 S 485 735 650 840 S 895 955 1080 850 S 1260 850 1300 1020',
    'M 1490 -100 C 1280 45 1345 145 1505 175 S 1710 330 1500 445 S 1335 650 1530 700 S 1710 875 1410 1000',
  ];
</script>

<div class="home-backdrop" aria-hidden="true">
  <svg class="roads" viewBox="0 0 1600 900" preserveAspectRatio="none" focusable="false">
    <g fill="none" stroke-linecap="round" stroke-linejoin="round">
      {#each roads as road}
        <path class="road-edge" d={road}/>
        <path class="road-surface" d={road}/>
        <path class="road-markings" d={road}/>
      {/each}
    </g>
  </svg>

  <svg class="signpost scenic-sign" viewBox="0 0 180 170" focusable="false">
    <g stroke="var(--ink)" stroke-width="3" stroke-linejoin="round">
      <path d="M88 52h8v107h-8z" fill="#9aaceb"/>
      <path d="M68 161h48" fill="none" stroke-linecap="round"/>
      <g transform="rotate(-8 90 42)">
        <path d="M9 21h137l24 22-24 22H9z" fill="var(--paper)"/>
        <circle cx="20" cy="43" r="2" fill="var(--ink)" stroke="none"/>
        <text x="34" y="48" stroke="none">SCENIC ROUTE</text>
      </g>
      <g transform="rotate(5 84 85)">
        <path d="M17 68h134v35H17L3 85z" fill="var(--yellow)"/>
        <text class="small-sign-text" x="26" y="90" stroke="none">TAKE THE LONG WAY</text>
      </g>
    </g>
  </svg>

  <svg class="signpost winding-sign" viewBox="0 0 100 155" focusable="false">
    <g stroke="var(--ink)" stroke-width="3" stroke-linejoin="round">
      <path d="M46 72h8v75h-8z" fill="#9aaceb"/>
      <path d="M33 149h34" fill="none" stroke-linecap="round"/>
      <g transform="rotate(10 50 49)">
        <rect x="18" y="17" width="64" height="64" rx="5" transform="rotate(45 50 49)" fill="var(--yellow)"/>
        <path d="M50 75V64c0-10 14-9 14-19S37 37 37 29c0-5 7-7 13-7m-6-5 6 5-6 6" fill="none" stroke-width="5" stroke-linecap="round"/>
      </g>
    </g>
  </svg>

  <svg class="signpost oddities-sign" viewBox="0 0 185 140" focusable="false">
    <g stroke="var(--ink)" stroke-width="3" stroke-linejoin="round">
      <path d="M88 42h8v86h-8z" fill="#9aaceb"/>
      <path d="M72 130h40" fill="none" stroke-linecap="round"/>
      <g transform="rotate(7 92 36)">
        <path d="M8 14h144l24 22-24 22H8z" fill="var(--paper)"/>
        <circle cx="20" cy="36" r="2" fill="var(--ink)" stroke="none"/>
        <text x="33" y="41" stroke="none">ODDITIES AHEAD</text>
      </g>
    </g>
  </svg>
</div>

<style>
  .home-backdrop {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    user-select: none;
  }

  .roads { width: 100%; height: 100%; }
  .roads path { vector-effect: non-scaling-stroke; }
  .road-edge { stroke: #b3c4ff; stroke-opacity: .3; stroke-width: 66px; }
  .road-surface { stroke: var(--blue-dark); stroke-opacity: .85; stroke-width: 62px; }
  .road-markings { stroke: var(--paper); stroke-opacity: .38; stroke-width: 2px; stroke-dasharray: 12 18; }

  .signpost { position: absolute; width: clamp(105px, 10vw, 170px); opacity: .78; }
  .signpost text { fill: var(--ink); font-family: var(--heading); font-size: 13px; font-weight: 800; letter-spacing: .3px; }
  .signpost .small-sign-text { font-size: 10px; }
  .scenic-sign { top: 6%; left: 6%; transform: rotate(-5deg); }
  .winding-sign { top: 9%; right: 10%; width: clamp(65px, 5.5vw, 90px); transform: rotate(4deg); }
  .oddities-sign { right: 5%; bottom: 8%; transform: rotate(5deg); }

  @media (max-width: 1100px), (max-height: 700px) {
    .signpost { opacity: .6; }
    .scenic-sign { top: 2%; left: 1%; width: 105px; }
    .winding-sign { top: 3%; right: 5%; width: 58px; }
    .oddities-sign { right: 2%; bottom: 4%; width: 110px; }
    .road-edge { stroke-width: 50px; }
    .road-surface { stroke-width: 46px; }
  }

  @media (max-width: 600px) {
    .signpost { display: none; }
    .home-backdrop { opacity: .65; }
    .road-edge { stroke-width: 34px; }
    .road-surface { stroke-width: 30px; }
    .road-markings { stroke-dasharray: 8 14; }
  }
</style>
