import { test, expect } from '@playwright/test';
import type { Room } from '../src/lib/api/engine';

interface PanoramaUpdate {
  event: string;
  visible?: boolean;
  status?: 'OK' | 'ZERO_RESULTS';
  panoramaId?: string;
  position?: { lat: number; lng: number } | null;
}

test('saving follows late Street View positions, navigation, and visibility', async ({ page }) => {
  // Exercise the live-map component without a Maps key or network requests to Google.
  await page.route('**/src/lib/maps/loader.ts*', route => route.fulfill({
    contentType: 'application/javascript',
    body: 'export const mapsConfigured = true; export async function loadMaps() {}',
  }));
  await page.addInitScript(() => {
    const listeners: Record<string, (() => void)[]> = {};
    const current = {
      visible: false, status: 'OK', panoramaId: 'panorama-1',
      position: null as { lat: number; lng: number } | null,
    };
    const emit = (name: string) => listeners[name]?.forEach(callback => callback());
    const panorama = {
      addListener(name: string, callback: () => void) { (listeners[name] ??= []).push(callback); },
      getVisible: () => current.visible,
      getStatus: () => current.status,
      getPano: () => current.panoramaId,
      getPosition: () => current.position ? { toJSON: () => ({ ...current.position }) } : null,
      getPov: () => ({ heading: 90, pitch: 0 }),
      getZoom: () => 1,
      setVisible(value: boolean) { current.visible = value; emit('visible_changed'); },
      setOptions() {},
      unbindAll() {},
    };
    window.addEventListener('test:panorama', event => {
      const { event: name, ...update } = (event as CustomEvent<PanoramaUpdate>).detail;
      Object.assign(current, update);
      emit(name);
    });
    Object.assign(window, { google: { maps: {
      Map: class {
        getStreetView() { return panorama; }
        unbindAll() {}
      },
      MapTypeId: { ROADMAP: 'roadmap', SATELLITE: 'satellite' },
      MapTypeControlStyle: { HORIZONTAL_BAR: 1 },
      ControlPosition: { TOP_RIGHT: 3 },
      StreetViewStatus: { OK: 'OK', ZERO_RESULTS: 'ZERO_RESULTS' },
      event: { trigger() {}, clearInstanceListeners() {} },
    } } });
  });

  await page.goto('/');
  await page.getByLabel('Your spotter name').fill('Ada');
  await page.locator('form').getByRole('button', { name: 'Create room', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await page.getByLabel('New category').fill('A blue door');
  await page.getByRole('button', { name: 'Add category', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Drag the yellow Pegman onto a blue street.')).toBeVisible();

  const save = page.getByRole('button', { name: /Save view/ });
  const update = (value: PanoramaUpdate) => page.evaluate(detail => {
    window.dispatchEvent(new CustomEvent('test:panorama', { detail }));
  }, value);
  const findings = () => page.evaluate(() => {
    const id = location.hash.split('/').at(-1);
    const room: Room = JSON.parse(localStorage.getItem(`sls:room:${id}`)!);
    return room.findings;
  });

  await expect(save).toBeDisabled();
  await update({ event: 'visible_changed', visible: true });
  await update({ event: 'pano_changed' });
  await update({ event: 'status_changed', status: 'OK' });
  await expect(save).toBeDisabled();

  // Position can arrive after visibility, panorama ID, and lookup status have settled.
  const firstPosition = { lat: 52.52, lng: 13.405 };
  await update({ event: 'position_changed', position: firstPosition });
  await expect(save).toBeEnabled();
  await save.click();
  await expect.poll(findings).toHaveLength(1);
  expect((await findings())[0].view).toEqual({
    panoramaId: 'panorama-1', position: firstPosition, heading: 90, pitch: 0, zoom: 1,
  });

  // The position can also change without changing the panorama ID.
  const secondPosition = { lat: 52.521, lng: 13.406 };
  await update({ event: 'position_changed', position: secondPosition });
  await expect(save).toBeEnabled();
  await save.click();
  await expect.poll(findings).toHaveLength(2);
  expect((await findings())[1].view.position).toEqual(secondPosition);

  await update({ event: 'pano_changed', panoramaId: 'panorama-2', position: null });
  await expect(save).toBeDisabled();
  await update({ event: 'position_changed', position: firstPosition });
  await expect(save).toBeEnabled();
  await update({ event: 'status_changed', status: 'ZERO_RESULTS' });
  await expect(save).toBeDisabled();
  await update({ event: 'status_changed', status: 'OK' });
  await expect(save).toBeEnabled();
  await page.getByRole('button', { name: 'Back to map' }).click();
  await expect(save).toBeDisabled();
});
