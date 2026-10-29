<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import Icon from '../components/Icon.svelte';
  import Scene from '../components/Scene.svelte';
  import type { Viewpoint } from '../api/types';
  import { loadMaps, mapsConfigured } from './loader';
  import { MapsSession, mapsSessionKey } from './session';

  export let explore = false;
  export let demo = false;
  export let view: Viewpoint | null = null;
  export let onview: (view: Viewpoint | null) => void = () => {};

  let element: HTMLDivElement;
  let panorama: google.maps.StreetViewPanorama | undefined;
  let map: google.maps.Map | undefined;
  let error = '';
  let loading = true;
  let scene = 0;
  let inStreet = false;
  let initialized = false;
  let applied = '';
  let lifecycle = 0;
  const unavailable = 'This Street View panorama is no longer available.';
  const maps = getContext<MapsSession>(mapsSessionKey);
  const illustrated = () => demo || view?.demoScene !== undefined;

  function demoChange(next: number) {
    scene = (next + 3) % 3;
    onview({ panoramaId: `demo-${scene}`, position: { lat: 52.52, lng: 13.405 }, heading: 90, pitch: 0, zoom: 1, demoScene: scene });
  }

  $: if (initialized && panorama && !explore && view && view.demoScene === undefined) {
    const key = JSON.stringify(view);
    if (key !== applied) {
      applied = key;
      const samePanorama = panorama.getPano() === view.panoramaId;
      error = samePanorama && panorama.getStatus() === google.maps.StreetViewStatus.ZERO_RESULTS ? unavailable : '';
      loading = !error && (!samePanorama || panorama.getStatus() !== google.maps.StreetViewStatus.OK);
      if (!samePanorama) panorama.setPano(view.panoramaId);
      panorama.setPov({ heading: view.heading, pitch: view.pitch });
      panorama.setZoom(view.zoom);
      panorama.setVisible(true);
    }
  }

  onMount(() => {
    const generation = ++lifecycle;
    let disposed = false;
    let observer: ResizeObserver | undefined;
    let canvas: HTMLDivElement | undefined;
    const listeners: google.maps.MapsEventListener[] = [];
    const lockInteraction = (event: Event) => {
      if (explore || (event.target instanceof Element && event.target.closest('a'))) return;
      if (event instanceof KeyboardEvent && (event.key === 'Tab' || event.key === 'Escape' || event.ctrlKey || event.metaKey || event.altKey)) return;
      event.preventDefault();
      event.stopImmediatePropagation();
    };
    const lockedEvents = ['pointerdown', 'mousedown', 'dblclick', 'click', 'keydown'];
    if (!explore && element) lockedEvents.forEach(name => element.addEventListener(name, lockInteraction, true));
    const authError = () => { error = 'Google Maps could not authenticate. Check the key, billing, API enablement, and website restrictions.'; loading = false; onview(null); };
    window.addEventListener('sls:maps-auth-error', authError);
    if (illustrated()) {
      loading = false;
      if (explore) demoChange(0);
    } else if (!mapsConfigured) {
      loading = false;
      onview(null);
    } else {
      void loadMaps().then(() => {
        if (disposed || generation !== lifecycle) return;
        const viewer = maps.get(explore);
        canvas = viewer.element;
        element.append(canvas);
        map = viewer.map;
        panorama = viewer.panorama;
        viewer.mapElement.hidden = !explore;
        panorama.setOptions({
          addressControl: false, fullscreenControl: false, motionTracking: false, motionTrackingControl: false,
          disableDefaultUI: !explore, clickToGo: explore, linksControl: explore,
          scrollwheel: explore, showRoadLabels: explore, enableCloseButton: explore,
        });
        if (explore) loading = false;
        const changed = () => {
          if (!panorama || disposed) return;
          inStreet = panorama.getVisible();
          if (panorama.getStatus() !== google.maps.StreetViewStatus.OK) {
            onview(null);
            return;
          }
          loading = false;
          if (explore) {
            const position = panorama.getPosition();
            onview(inStreet && position ? { panoramaId: panorama.getPano(), position: position.toJSON(), ...panorama.getPov(), zoom: panorama.getZoom() } : null);
          }
        };
        // Position can settle after the pano/status events or change without a new pano ID.
        ['pano_changed', 'position_changed', 'pov_changed', 'zoom_changed', 'visible_changed'].forEach(event => listeners.push(panorama!.addListener(event, changed)));
        listeners.push(panorama.addListener('status_changed', () => {
          if (panorama?.getStatus() === google.maps.StreetViewStatus.ZERO_RESULTS) {
            error = unavailable; loading = false; onview(null);
          } else if (panorama?.getStatus() === google.maps.StreetViewStatus.OK) { error = ''; changed(); }
        }));
        observer = new ResizeObserver(() => {
          if (map) google.maps.event.trigger(map, 'resize');
          if (panorama) google.maps.event.trigger(panorama, 'resize');
        });
        observer.observe(element);
        if (map) google.maps.event.trigger(map, 'resize');
        google.maps.event.trigger(panorama, 'resize');
        initialized = true;
        changed();
      }).catch(cause => { if (!disposed) { error = (cause as Error).message; loading = false; } });
    }
    return () => {
      disposed = true;
      lifecycle++;
      onview(null);
      observer?.disconnect();
      if (element) lockedEvents.forEach(name => element.removeEventListener(name, lockInteraction, true));
      window.removeEventListener('sls:maps-auth-error', authError);
      listeners.forEach(listener => listener.remove());
      panorama?.setVisible(false);
      canvas?.remove();
    };
  });
</script>

<div class="street-view" class:locked={!explore}>
  {#if illustrated()}
    <Scene scene={explore ? scene : view?.demoScene ?? 0}/>
    <span class="map-label"><Icon name="spark" size={15}/> Illustrated demo · not Google Street View</span>
    {#if explore}
      <div class="demo-scenes" aria-label="Demo street scenes">
        <button class="map-button" onclick={() => demoChange(scene - 1)} aria-label="Previous demo scene"><span class="rotate-arrow"><Icon name="arrow" size={17}/></span></button>
        <span>Sample street {scene + 1} / 3</span>
        <button class="map-button" onclick={() => demoChange(scene + 1)} aria-label="Next demo scene"><Icon name="arrow" size={17}/></button>
      </div>
    {/if}
  {:else}
    <div bind:this={element} class="google-canvas" role="region" aria-label={explore ? 'Explore Google Maps and Street View' : 'Saved Street View with camera locked'}></div>
    {#if !mapsConfigured || error}
      <div class="map-empty">
        <span class="round-icon"><Icon name="map" size={32}/></span>
        <h3>{error ? 'A small detour.' : 'Your window to the world.'}</h3>
        <p>{error || 'Add your Google Maps browser key to enable the live map and Street View.'}</p>
        {#if !error}<code>frontend/.env.local<br/>VITE_GOOGLE_MAPS_API_KEY=…</code><p class="small">Restart Vite after adding the key. You can explore the solo demo without one.</p>{/if}
      </div>
    {:else if loading}
      <div class="map-empty"><span class="spinner"></span><p>Opening your view…</p></div>
    {/if}
    {#if explore && mapsConfigured && !error && !loading}
      <span class="map-label"><Icon name={inStreet ? 'eye' : 'pin'} size={16}/>{inStreet ? 'Find your frame, then save it to a category.' : 'Drag the yellow Pegman onto a blue street.'}</span>
      {#if inStreet}<button class="map-button back-to-map" onclick={() => { panorama?.setVisible(false); onview(null); }}><Icon name="map" size={16}/> Back to map</button>{/if}
    {/if}
  {/if}
</div>
