// A player's match owns the Google objects; individual screens only borrow them.
export const mapsSessionKey = Symbol('maps-session');
const fallbackCenter = { lat: 52.52, lng: 13.405 };
// Unverified start points are town centres; the nearest outdoor panorama is usually close.
const startSearchRadius = 5000;

export class MapsSession {
  private viewer?: {
    element: HTMLDivElement;
    mapElement: HTMLDivElement;
    panorama: google.maps.StreetViewPanorama;
    map?: google.maps.Map;
  };

  /** `start` positions a newly created explore map and opens its nearest panorama once. */
  get(explore: boolean, start?: google.maps.LatLngLiteral | null) {
    if (!this.viewer) {
      const element = document.createElement('div');
      element.className = 'google-canvas';
      element.style.position = 'relative';
      const mapElement = document.createElement('div');
      const panoramaElement = document.createElement('div');
      for (const canvas of [mapElement, panoramaElement]) {
        canvas.style.position = 'absolute';
        canvas.style.inset = '0';
        element.append(canvas);
      }
      const panorama = new google.maps.StreetViewPanorama(panoramaElement, { visible: false });
      panoramaElement.hidden = true;
      panorama.addListener('visible_changed', () => { panoramaElement.hidden = !panorama.getVisible(); });
      this.viewer = { element, mapElement, panorama };
    }
    const viewer = this.viewer;
    if (explore && !viewer.map) {
      viewer.map = new google.maps.Map(viewer.mapElement, {
        center: start ?? fallbackCenter, zoom: start ? 15 : 14,
        mapTypeControl: true,
        mapTypeControlOptions: {
          mapTypeIds: [google.maps.MapTypeId.ROADMAP, google.maps.MapTypeId.SATELLITE],
          style: google.maps.MapTypeControlStyle.HORIZONTAL_BAR,
          position: google.maps.ControlPosition.TOP_RIGHT,
        },
        fullscreenControl: false, streetViewControl: true, clickableIcons: false,
        streetView: viewer.panorama,
      });
      if (start) this.openNearestPanorama(start);
    }
    return viewer;
  }

  // One Street View lookup per player and round; the map stays centred on failure.
  private openNearestPanorama(start: google.maps.LatLngLiteral) {
    const viewer = this.viewer!;
    void new google.maps.StreetViewService().getPanorama({
      location: start, radius: startSearchRadius,
      preference: google.maps.StreetViewPreference.NEAREST,
      sources: [google.maps.StreetViewSource.GOOGLE, google.maps.StreetViewSource.OUTDOOR],
    }).then(({ data }) => {
      const pano = data.location?.pano;
      // Respect a player who already dropped Pegman somewhere else.
      if (!pano || this.viewer !== viewer || viewer.panorama.getVisible()) return;
      viewer.panorama.setPano(pano);
      viewer.panorama.setVisible(true);
    }).catch(() => { /* No coverage nearby: the player starts on the map instead. */ });
  }

  dispose() {
    if (!this.viewer) return;
    const { element, map, panorama } = this.viewer;
    google.maps.event.clearInstanceListeners(panorama);
    panorama.setVisible(false);
    panorama.unbindAll();
    if (map) {
      google.maps.event.clearInstanceListeners(map);
      map.unbindAll();
    }
    element.remove();
    this.viewer = undefined;
  }
}
