// A player's match owns the Google objects; individual screens only borrow them.
export const mapsSessionKey = Symbol('maps-session');

export class MapsSession {
  private viewer?: {
    element: HTMLDivElement;
    mapElement: HTMLDivElement;
    panorama: google.maps.StreetViewPanorama;
    map?: google.maps.Map;
  };

  get(explore: boolean) {
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
        center: { lat: 52.52, lng: 13.405 }, zoom: 14,
        mapTypeControl: true,
        mapTypeControlOptions: {
          mapTypeIds: [google.maps.MapTypeId.ROADMAP, google.maps.MapTypeId.SATELLITE],
          style: google.maps.MapTypeControlStyle.HORIZONTAL_BAR,
          position: google.maps.ControlPosition.TOP_RIGHT,
        },
        fullscreenControl: false, streetViewControl: true, clickableIcons: false,
        streetView: viewer.panorama,
      });
    }
    return viewer;
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
