import { test, expect, type Page } from '@playwright/test';
import type { Room } from '../src/lib/api/engine';

interface PanoramaUpdate {
  event: string;
  visible?: boolean;
  status?: 'OK' | 'ZERO_RESULTS';
  panoramaId?: string;
  position?: { lat: number; lng: number } | null;
  heading?: number;
  pitch?: number;
  zoom?: number;
}

test.beforeEach(async ({ page }) => {
  // Exercise the live-map component without a Maps key or network requests to Google.
  await page.route('**/src/lib/maps/loader.ts*', route => route.fulfill({
    contentType: 'application/javascript',
    body: 'export const mapsConfigured = true; export async function loadMaps() {}',
  }));
  await page.addInitScript(() => {
    const listeners: Record<string, Set<() => void>> = {};
    const stats = {
      maps: 0, panoramas: 0, cleared: 0, unbound: 0,
      options: {} as Record<string, unknown>, panoCalls: [] as string[],
    };
    const current = {
      visible: false, status: 'OK', panoramaId: 'panorama-1',
      position: null as { lat: number; lng: number } | null,
      heading: 90, pitch: 0, zoom: 1,
    };
    const emit = (name: string) => listeners[name]?.forEach(callback => callback());
    const panorama = {
      addListener(name: string, callback: () => void) {
        (listeners[name] ??= new Set()).add(callback);
        return { remove: () => listeners[name].delete(callback) };
      },
      getVisible: () => current.visible,
      getStatus: () => current.status,
      getPano: () => current.panoramaId,
      getPosition: () => current.position ? { toJSON: () => ({ ...current.position }) } : null,
      getPov: () => ({ heading: current.heading, pitch: current.pitch }),
      getZoom: () => current.zoom,
      setPano(value: string) { stats.panoCalls.push(value); current.panoramaId = value; current.status = 'OK'; emit('pano_changed'); },
      setPov(value: { heading: number; pitch: number }) { Object.assign(current, value); emit('pov_changed'); },
      setZoom(value: number) { current.zoom = value; emit('zoom_changed'); },
      setVisible(value: boolean) { current.visible = value; emit('visible_changed'); },
      setOptions(options: Record<string, unknown>) { Object.assign(stats.options, options); },
      unbindAll() { stats.unbound++; },
    };
    window.addEventListener('test:panorama', event => {
      const { event: name, ...update } = (event as CustomEvent<PanoramaUpdate>).detail;
      Object.assign(current, update);
      emit(name);
    });
    window.addEventListener('test:maps-stats', () => {
      Object.assign(window, { mapsStats: {
        ...stats, current: { ...current },
        listeners: Object.values(listeners).reduce((sum, callbacks) => sum + callbacks.size, 0),
      } });
    });
    Object.assign(window, { google: { maps: {
      Map: class {
        constructor(element: HTMLElement, options: { streetView: unknown }) {
          stats.maps++;
          if (options.streetView !== panorama) throw new Error('Map must use the shared panorama');
          element.dataset.testMap = String(stats.maps);
        }
        getStreetView() { return panorama; }
        unbindAll() { stats.unbound++; }
      },
      StreetViewPanorama: class {
        constructor(element: HTMLElement) {
          stats.panoramas++;
          element.dataset.testPanorama = String(stats.panoramas);
          return panorama;
        }
      },
      MapTypeId: { ROADMAP: 'roadmap', SATELLITE: 'satellite' },
      MapTypeControlStyle: { HORIZONTAL_BAR: 1 },
      ControlPosition: { TOP_RIGHT: 3 },
      StreetViewStatus: { OK: 'OK', ZERO_RESULTS: 'ZERO_RESULTS' },
      event: {
        trigger() {},
        clearInstanceListeners(target: unknown) {
          stats.cleared++;
          if (target === panorama) Object.values(listeners).forEach(callbacks => callbacks.clear());
        },
      },
    } } });
  });
});

async function startSearch(page: Page, navigate = true) {
  if (navigate) await page.goto('/');
  await page.getByLabel('Your spotter name').fill('Ada');
  await page.getByRole('group', { name: 'Session action' }).getByRole('button', { name: 'Create room', exact: true }).click();
  await page.locator('form').getByRole('button', { name: 'Create room', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await page.getByLabel('New category').fill('A blue door');
  await page.getByRole('button', { name: 'Add category', exact: true }).click();
  await page.getByLabel('New category').fill('A bicycle');
  await page.getByRole('button', { name: 'Add category', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Drag the yellow Pegman onto a blue street.')).toBeVisible();
}

const update = (page: Page, value: PanoramaUpdate) => page.evaluate(detail => {
  window.dispatchEvent(new CustomEvent('test:panorama', { detail }));
}, value);

async function stats(page: Page) {
  return page.evaluate(() => {
    window.dispatchEvent(new Event('test:maps-stats'));
    return (window as unknown as { mapsStats: {
      maps: number; panoramas: number; cleared: number; unbound: number; listeners: number;
      options: Record<string, unknown>; panoCalls: string[];
      current: { panoramaId: string; visible: boolean; heading: number; pitch: number; zoom: number };
    } }).mapsStats;
  });
}

async function endSearch(page: Page, addEmptyCategory = false) {
  await page.evaluate(addEmpty => {
    const id = location.hash.split('/').at(-1)!;
    const room: Room = JSON.parse(localStorage.getItem(`sls:room:${id}`)!);
    if (addEmpty) room.categories.push({ id: 'empty', text: 'Nothing saved' });
    room.searchEndsAt = Date.now() - 1;
    room.revision++;
    localStorage.setItem(`sls:room:${id}`, JSON.stringify(room));
    window.dispatchEvent(new CustomEvent('sls:changed', { detail: id }));
  }, addEmptyCategory);
}

test('saving follows late Street View positions, navigation, and visibility', async ({ page }) => {
  await startSearch(page);

  const save = page.getByRole('button', { name: /Save view/ });
  const findings = () => page.evaluate(() => {
    const id = location.hash.split('/').at(-1);
    const room: Room = JSON.parse(localStorage.getItem(`sls:room:${id}`)!);
    return room.findings;
  });

  await expect(save).toBeDisabled();
  await update(page, { event: 'visible_changed', visible: true });
  await update(page, { event: 'pano_changed' });
  await update(page, { event: 'status_changed', status: 'OK' });
  await expect(save).toBeDisabled();

  // Position can arrive after visibility, panorama ID, and lookup status have settled.
  const firstPosition = { lat: 52.52, lng: 13.405 };
  await update(page, { event: 'position_changed', position: firstPosition });
  await expect(save).toBeEnabled();
  await save.click();
  await expect.poll(findings).toHaveLength(1);
  expect((await findings())[0].view).toEqual({
    panoramaId: 'panorama-1', position: firstPosition, heading: 90, pitch: 0, zoom: 1,
  });

  // The position can also change without changing the panorama ID.
  const secondPosition = { lat: 52.521, lng: 13.406 };
  await update(page, { event: 'position_changed', position: secondPosition });
  await expect(save).toBeEnabled();
  await save.click();
  await expect.poll(findings).toHaveLength(2);
  expect((await findings())[1].view.position).toEqual(secondPosition);

  await update(page, { event: 'pano_changed', panoramaId: 'panorama-2', position: null });
  await expect(save).toBeDisabled();
  await update(page, { event: 'position_changed', position: firstPosition });
  await expect(save).toBeEnabled();
  await update(page, { event: 'status_changed', status: 'ZERO_RESULTS' });
  await expect(save).toBeDisabled();
  await update(page, { event: 'status_changed', status: 'OK' });
  await expect(save).toBeEnabled();
  await page.getByRole('button', { name: 'Back to map' }).click();
  await expect(save).toBeDisabled();
});

test('one map and panorama serve search, empty categories, candidate changes, and all review items', async ({ page }) => {
  await startSearch(page);
  await update(page, { event: 'position_changed', visible: true, position: { lat: 52.52, lng: 13.405 } });
  const save = page.getByRole('button', { name: /Save view/ });
  await save.click();
  await update(page, { event: 'pano_changed', panoramaId: 'panorama-2', heading: 135, pitch: 12, zoom: 2 });
  await save.click();
  await page.getByRole('button', { name: /A bicycle/ }).click();
  await update(page, { event: 'pano_changed', panoramaId: 'panorama-3', heading: 250, pitch: -5, zoom: 3 });
  await save.click();
  await endSearch(page, true);
  await expect(page.getByRole('heading', { name: /your best shot/i })).toBeVisible();
  await expect.poll(async () => (await stats(page)).current).toMatchObject({ panoramaId: 'panorama-1', heading: 90, pitch: 0, zoom: 1 });
  expect(await stats(page)).toMatchObject({ maps: 1, panoramas: 1, cleared: 0, unbound: 0, options: { disableDefaultUI: true, clickToGo: false, enableCloseButton: false } });
  await page.getByRole('button', { name: /View 2 Choose this view instead/ }).click();
  await expect.poll(async () => (await stats(page)).current).toMatchObject({ panoramaId: 'panorama-2', heading: 135, pitch: 12, zoom: 2 });
  await update(page, { event: 'status_changed', status: 'ZERO_RESULTS' });
  await expect(page.getByText('This Street View panorama is no longer available.')).toBeVisible();

  // The canvas itself survives periods without a StreetView component.
  await page.getByRole('button', { name: /Nothing saved/ }).click();
  await expect(page.getByRole('heading', { name: 'No finding, no fuss.' })).toBeVisible();
  expect(await stats(page)).toMatchObject({ maps: 1, panoramas: 1, cleared: 0 });
  await page.getByRole('button', { name: /A blue door/ }).click();
  await expect(page.getByText('This Street View panorama is no longer available.')).toBeVisible();
  await page.getByRole('button', { name: /A bicycle/ }).click();
  await expect.poll(async () => (await stats(page)).current).toMatchObject({ panoramaId: 'panorama-3', heading: 250, pitch: -5, zoom: 3 });
  expect(await stats(page)).toMatchObject({ listeners: 7 });
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Finding 1 of 2')).toBeVisible();
  await expect.poll(async () => (await stats(page)).current).toMatchObject({ panoramaId: 'panorama-2', heading: 135, pitch: 12, zoom: 2 });
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Finding 2 of 2')).toBeVisible();
  await expect.poll(async () => (await stats(page)).current).toMatchObject({ panoramaId: 'panorama-3', heading: 250, pitch: -5, zoom: 3 });
  expect(await stats(page)).toMatchObject({ maps: 1, panoramas: 1, cleared: 0, unbound: 0, listeners: 7 });
  await expect(page.locator('[data-test-panorama="1"]')).toBeVisible();
  await expect(page.locator('[data-test-map="1"]')).toBeHidden();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The final tally' })).toBeVisible();
  expect(await stats(page)).toMatchObject({ maps: 1, panoramas: 1, listeners: 1, cleared: 0 });

  await page.getByRole('button', { name: 'Leave room', exact: true }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Leave room', exact: true }).click();
  await expect(page.getByLabel('Your spotter name')).toBeVisible();
  expect(await stats(page)).toMatchObject({ cleared: 2, unbound: 2, listeners: 0, current: { visible: false } });
  await startSearch(page, false);
  expect(await stats(page)).toMatchObject({ maps: 2, panoramas: 2 });
});

test('resuming directly into review creates only a panorama and reuses it', async ({ page }) => {
  await startSearch(page);
  await update(page, { event: 'position_changed', visible: true, position: { lat: 52.52, lng: 13.405 } });
  await page.getByRole('button', { name: /Save view/ }).click();
  await page.getByRole('button', { name: /A bicycle/ }).click();
  await update(page, { event: 'pano_changed', panoramaId: 'panorama-2', heading: 180 });
  await page.getByRole('button', { name: /Save view/ }).click();
  await endSearch(page);
  await expect(page.getByRole('heading', { name: /your best shot/i })).toBeVisible();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Finding 1 of 2')).toBeVisible();
  await page.reload();
  await expect(page.getByText('Finding 1 of 2')).toBeVisible();
  await expect.poll(async () => (await stats(page)).panoramas).toBe(1);
  expect(await stats(page)).toMatchObject({ maps: 0, panoramas: 1 });
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Finding 2 of 2')).toBeVisible();
  await expect.poll(async () => (await stats(page)).current).toMatchObject({ panoramaId: 'panorama-2', heading: 180 });
  expect(await stats(page)).toMatchObject({ maps: 0, panoramas: 1, cleared: 0, listeners: 7 });
});
