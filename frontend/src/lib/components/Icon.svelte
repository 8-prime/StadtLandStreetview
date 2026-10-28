<script lang="ts">
  export let name = 'pin';
  export let size = 20;
  const paths: Record<string, string> = {
    pin: 'M20 10c0 6-8 12-8 12S4 16 4 10a8 8 0 1 1 16 0ZM12 7a3 3 0 1 0 0 6 3 3 0 0 0 0-6Z',
    arrow: 'M4 12h15m-6-6 6 6-6 6',
    check: 'm5 12 4 4L19 6',
    plus: 'M12 5v14M5 12h14',
    close: 'm6 6 12 12M18 6 6 18',
    clock: 'M12 8v5l3 2M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z',
    users: 'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2m20 0v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75M13 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
    camera: 'M14 4H10L8 7H3v13h18V7h-5l-2-3ZM16 13a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z',
    compass: 'M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Zm-6-4-3 5-5 3 3-5 5-3Z',
    copy: 'M9 9h12v12H9V9Zm6-4V3H3v12h2',
    trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7',
    edit: 'm15 4 5 5M4 20l5-1L21 7l-5-5L4 14v6Z',
    trophy: 'M8 3h8v7a4 4 0 0 1-8 0V3Zm0 2H3v3a5 5 0 0 0 5 5m8-8h5v3a5 5 0 0 1-5 5m-4 1v6m-5 1h10',
    map: 'm2 6 7-3 6 3 7-3v15l-7 3-6-3-7 3V6Zm7-3v15m6-12v15',
    exit: 'M9 3H3v18h6m5-15 6 6-6 6m-6-6h12',
    info: 'M12 11v6m0-10v.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z',
    spark: 'm12 2 3 7 7 3-7 3-3 7-3-7-7-3 7-3 3-7Z',
    eye: 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Zm13 0a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z',
  };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={paths[name] || paths.pin} /></svg>
