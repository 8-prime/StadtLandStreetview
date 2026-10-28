import { importLibrary, setOptions } from '@googlemaps/js-api-loader';

export const mapsConfigured = Boolean(import.meta.env.VITE_GOOGLE_MAPS_API_KEY?.trim());
let loading: Promise<void> | undefined;

export function loadMaps() {
  if (!loading) {
    if (!mapsConfigured) return Promise.reject(new Error('Add VITE_GOOGLE_MAPS_API_KEY to frontend/.env.local and restart the development server.'));
    setOptions({ key: import.meta.env.VITE_GOOGLE_MAPS_API_KEY, v: 'weekly' });
    window.gm_authFailure = () => window.dispatchEvent(new Event('sls:maps-auth-error'));
    loading = Promise.all([importLibrary('maps'), importLibrary('streetView')]).then(() => undefined);
    loading.catch(() => { loading = undefined; });
  }
  return loading;
}

declare global { interface Window { gm_authFailure?: () => void } }
