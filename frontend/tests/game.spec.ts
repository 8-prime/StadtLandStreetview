import { test, expect, type Page } from '@playwright/test';
import type { Room } from '../src/lib/api/engine';
import type { Phase } from '../src/lib/api/types';

async function state(page: Page) {
  return page.evaluate(() => {
    const id = location.hash.split('/').at(-1);
    return JSON.parse(localStorage.getItem(`sls:room:${id}`)!);
  });
}
async function readyAndAdvance(page: Page) {
  const stepId = (await state(page)).stepId;
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect.poll(async () => (await state(page)).stepId).not.toBe(stepId);
}

async function capture(page: Page, name: string) {
  await page.evaluate(async () => {
    await document.fonts.ready;
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
  });
  await assertGameViewport(page);
  await page.screenshot({ path: `test-results/${name}.png` });
}

async function assertGameViewport(page: Page) {
  const original = page.viewportSize()!;
  for (const viewport of [{ width: 1280, height: 720 }, { width: 1440, height: 900 }, { width: 1920, height: 1080 }]) {
    await page.setViewportSize(viewport);
    await expect.poll(() => page.evaluate(() => {
      const problems: string[] = [];
      const root = document.documentElement;
      if (root.scrollWidth > innerWidth || root.scrollHeight > innerHeight || scrollX || scrollY) problems.push('Document is scrollable');
      for (const element of document.querySelectorAll<HTMLElement>('button, input, select, [data-spotter]')) {
        const rect = element.getBoundingClientRect();
        if (!rect.width || !rect.height || element.closest('dialog:not([open])')) continue;
        // Decorative art can rotate beyond its own box; all interactive controls must fit.
        if (element.matches('[data-spotter]')) continue;
        const label = element.getAttribute('aria-label') || element.textContent?.trim().slice(0, 45) || element.id;
        if (rect.top < -1 || rect.left < -1 || rect.bottom > innerHeight + 1 || rect.right > innerWidth + 1) problems.push(`Offscreen control: ${label}`);
        for (let parent = element.parentElement; parent; parent = parent.parentElement) {
          const style = getComputedStyle(parent);
          if (!['hidden', 'clip', 'auto', 'scroll'].includes(style.overflowY)) continue;
          const bounds = parent.getBoundingClientRect();
          if (rect.top < bounds.top - 3 || rect.bottom > bounds.bottom + 3) problems.push(`Clipped control: ${label}`);
        }
      }
      for (const element of document.querySelectorAll<HTMLElement>('*')) {
        if (!element.getClientRects().length || element.closest('dialog:not([open])')) continue;
        const style = getComputedStyle(element);
        if (['auto', 'scroll'].includes(style.overflowY) && element.scrollHeight > element.clientHeight + 2) problems.push(`Nested scroll region: ${element.className}`);
      }
      for (const frame of document.querySelectorAll<HTMLElement>('.view-frame')) {
        const rect = frame.getBoundingClientRect();
        if (Math.abs(rect.width / rect.height - 1.6) > .02) problems.push('Saved camera framing is not 16:10');
      }
      return [...new Set(problems)];
    })).toEqual([]);
  }
  await page.setViewportSize(original);
}

test('solo demo runs from lobby through candidate selection and all review results', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await page.getByRole('button', { name: /Try the solo demo/ }).click();
  await expect(page.getByRole('heading', { name: 'The curious crew' })).toBeVisible();
  await capture(page, 'lobby');
  await readyAndAdvance(page);
  await capture(page, 'categories');
  await page.getByRole('button', { name: 'Remove A car parked illegally', exact: true }).click();
  await page.getByRole('button', { name: 'Remove Something that makes you smile', exact: true }).click();
  await readyAndAdvance(page);
  await expect(page.getByRole('button', { name: /Save view/ })).toBeEnabled();
  await page.getByRole('button', { name: /Save view/ }).click();
  await page.getByRole('button', { name: 'Next demo scene' }).click();
  await page.getByRole('button', { name: /Save view/ }).click();
  await capture(page, 'search');
  await page.getByRole('button', { name: /Demo: end search/ }).click();
  await expect(page.getByRole('heading', { name: /your best shot/i })).toBeVisible();
  await expect(page.getByRole('button', { name: /View 1 Your final submission/ })).toHaveAttribute('aria-pressed', 'true');
  await page.getByRole('button', { name: /View 2 Choose this view instead/ }).click();
  await expect(page.getByRole('button', { name: /View 2 Your final submission/ })).toHaveAttribute('aria-pressed', 'true');
  await page.reload();
  await expect(page.getByRole('button', { name: /View 2 Your final submission/ })).toHaveAttribute('aria-pressed', 'true');
  await capture(page, 'selection');
  await readyAndAdvance(page);
  await expect(page.getByText('A moment for your find.')).toBeVisible();
  for (let index = 0; index < 3; index++) {
    if (index > 0) {
      await expect(page.getByRole('button', { name: 'I’m ready', exact: true })).toBeDisabled();
      if (index === 1) await capture(page, 'review');
      await page.getByRole('button', { name: 'Matches', exact: true }).click();
    }
    await readyAndAdvance(page);
    await expect(page.getByText('THE GROUP HAS SPOKEN')).toBeVisible();
    if (index === 0) await capture(page, 'result-reveal');
    await readyAndAdvance(page);
  }
  await expect(page.getByRole('heading', { name: 'The final tally' })).toBeVisible();
  expect(errors).toEqual([]);
  await capture(page, 'results');
});

test('two peers share readiness, category edits reset agreement, and refresh resumes identity', async ({ page, context }) => {
  await page.goto('/');
  await page.getByLabel('Your spotter name').fill('Ada');
  await page.getByLabel('Time on the streets').selectOption('60');
  await page.locator('form').getByRole('button', { name: 'Create room', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The curious crew' })).toBeVisible();
  const second = await context.newPage();
  await second.goto(page.url());
  await second.getByLabel('Your spotter name').fill('Linus');
  await second.locator('form').getByRole('button', { name: 'Join room', exact: true }).click();
  await expect(page.getByText('Linus', { exact: true })).toBeVisible();
  const adaId = (await state(page)).players.find((p: { name: string }) => p.name === 'Ada').id;
  const selector = `[data-spotter="${adaId}"]`;
  expect(await page.locator(selector).first().innerHTML()).toBe(await second.locator(selector).first().innerHTML());
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Ready · undo' })).toBeVisible();
  await page.getByRole('button', { name: 'Ready · undo' }).click();
  await expect(second.getByText('0 of 2 ready')).toBeVisible();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await page.getByLabel('New category').fill('A blue door');
  await page.getByRole('button', { name: 'Add category', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByLabel('New category').fill('A bicycle');
  await second.getByRole('button', { name: 'Add category', exact: true }).click();
  await expect(page.getByRole('button', { name: 'I’m ready', exact: true })).toBeVisible();
  await page.reload();
  await expect(page.getByText('0 of 2 ready')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Your scavenger list' })).toBeVisible();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByRole('button', { name: /Save view/ })).toBeDisabled();
  await expect(page.getByText('Your window to the world.')).toBeVisible();
  expect((await state(page)).phase).toBe('searching');
  // Inject camera output through the adapter: no Google key or imagery is needed
  // to verify that two actual UI peers see exactly the same review item.
  for (const peer of [page, second]) {
    await peer.evaluate(async () => {
      const modulePath = '/src/lib/api/index.ts';
      const { api } = await import(modulePath);
      const id = location.hash.split('/').at(-1)!;
      const session = await api.get(id);
      await api.send(id, { type: 'save-finding', categoryId: session.categories[0].id, view: { panoramaId: 'test-panorama', position: { lat: 52.52, lng: 13.405 }, heading: 90, pitch: 0, zoom: 1 } });
    });
  }
  await page.clock.install();
  await page.clock.fastForward(62000);
  await expect(page.getByRole('heading', { name: /your best shot/i })).toBeVisible();
  await expect(second.getByRole('heading', { name: /your best shot/i })).toBeVisible();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('A moment for your find.')).toBeVisible();
  await expect(second.getByText('A match, or a stretch?')).toBeVisible();
  await second.getByRole('button', { name: 'Doesn’t match', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('Not quite a match.')).toBeVisible();
  await expect(second.getByText('Not quite a match.')).toBeVisible();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('A match, or a stretch?')).toBeVisible();
  await expect(second.getByText('A moment for your find.')).toBeVisible();
  await page.getByRole('button', { name: 'Matches', exact: true }).click();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByText('That’s a match.')).toBeVisible();
  await page.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await second.getByRole('button', { name: 'I’m ready', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The final tally' })).toBeVisible();
  await expect(second.getByRole('heading', { name: 'The final tally' })).toBeVisible();
  expect((await state(page)).results).toHaveLength(2);
  await page.clock.resume();
  await second.getByRole('button', { name: 'Leave room', exact: true }).click();
  await second.getByRole('dialog').getByRole('button', { name: 'Leave room', exact: true }).click();
  await expect.poll(async () => (await state(page)).players.find((p: { name: string }) => p.name === 'Linus').left).toBe(true);
});

test('home and rules are usable with keyboard and fit a desktop viewport', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'How to play' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await assertGameViewport(page);
  for (let step = 1; step < 4; step++) {
    await page.getByRole('button', { name: 'Next rules page' }).click();
    await assertGameViewport(page);
  }
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).not.toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await capture(page, 'home');
});

test('maximum crew, long categories, and twelve candidates remain accessible without scrolling', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto('/');
  await page.getByLabel('Your spotter name').fill('The extremely nosy one');
  await page.locator('form').getByRole('button', { name: 'Create room', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The curious crew' })).toBeVisible();
  await page.evaluate(() => {
    const id = location.hash.split('/').at(-1)!;
    const room: Room = JSON.parse(localStorage.getItem(`sls:room:${id}`)!);
    const me = room.players[0];
    room.demo = true;
    room.players = Array.from({ length: 12 }, (_, i) => ({ ...me, id: i ? `fixture-spotter-${i}` : me.id, name: `Spotter_${i + 1}_${'x'.repeat(24)}`.slice(0, 24), color: i % 6, ready: false, bot: false, score: i % 3 }));
    room.categories = Array.from({ length: 12 }, (_, i) => ({ id: `category-${i}`, text: `Assignment ${i + 1}: ${'something suspiciously specific '.repeat(4)}`.slice(0, 100) }));
    const view = { panoramaId: 'fixture', position: { lat: 52.52, lng: 13.405 }, heading: 90, pitch: 0, zoom: 1, demoScene: 0 };
    room.findings = Array.from({ length: 12 }, (_, i) => ({ id: `candidate-${i}`, playerId: me.id, categoryId: 'category-0', view: { ...view, demoScene: i % 3 }, createdAt: Date.now() }));
    for (const player of room.players.slice(1)) room.findings.push({ id: `finding-${player.id}`, playerId: player.id, categoryId: 'category-0', view, createdAt: Date.now() });
    room.selections = { [me.id]: { 'category-0': 'candidate-0' } };
    room.queue = room.findings.map(f => f.id);
    room.revision++;
    localStorage.setItem(`sls:room:${id}`, JSON.stringify(room));
    window.dispatchEvent(new CustomEvent('sls:changed', { detail: id }));
  });
  async function phase(value: Phase) {
    await page.evaluate(next => {
      const id = location.hash.split('/').at(-1)!;
      const room: Room = JSON.parse(localStorage.getItem(`sls:room:${id}`)!);
      room.phase = next;
      room.stepId = crypto.randomUUID();
      room.searchEndsAt = Date.now() + 600000;
      room.reviewIndex = 12;
      room.revealed = false;
      if (next === 'finished') room.results = [{ itemId: 'candidate-0', categoryId: 'category-0', playerId: room.players[0].id, accepted: true, accepts: 8, rejects: 3 }];
      room.revision++;
      localStorage.setItem(`sls:room:${id}`, JSON.stringify(room));
      window.dispatchEvent(new CustomEvent('sls:changed', { detail: id }));
    }, value);
    await expect(page.locator(`.phase-${value}`)).toBeVisible();
    await assertGameViewport(page);
  }
  await expect(page.locator('.player-card')).toHaveCount(12);
  await assertGameViewport(page);
  await page.screenshot({ path: 'test-results/full-crew-720.png' });
  await phase('categories');
  await page.getByRole('button', { name: 'Next categories page' }).click();
  await page.getByRole('button', { name: 'Next categories page' }).click();
  await expect(page.locator('.category-list li')).toHaveCount(4);
  await expect(page.getByRole('button', { name: 'Next categories page' })).toBeDisabled();
  await assertGameViewport(page);
  await page.getByRole('button', { name: /^Edit Assignment 9:/ }).click();
  await assertGameViewport(page);
  await phase('searching');
  for (let i = 0; i < 3; i++) await page.getByRole('button', { name: 'Next saved views page' }).click();
  await expect(page.getByRole('button', { name: 'Remove saved view 12', exact: true })).toBeVisible();
  await assertGameViewport(page);
  await phase('selection');
  for (let i = 0; i < 3; i++) await page.getByRole('button', { name: 'Next candidates page' }).click();
  await page.getByRole('button', { name: /View 12 Choose this view instead/ }).click();
  await expect(page.getByRole('button', { name: /View 12 Your final submission/ })).toHaveAttribute('aria-pressed', 'true');
  await assertGameViewport(page);
  await page.getByRole('button', { name: 'Next checklist page' }).click();
  await page.getByRole('button', { name: 'Next checklist page' }).click();
  await page.getByRole('button', { name: /^12 Assignment 12:/ }).click();
  await expect(page.getByRole('heading', { name: 'No finding, no fuss.' })).toBeVisible();
  await assertGameViewport(page);
  await phase('review');
  await expect(page.getByRole('button', { name: 'Matches', exact: true })).toBeVisible();
  await assertGameViewport(page);
  await phase('finished');
  await expect(page.locator('.score-row')).toHaveCount(6);
  await page.getByRole('button', { name: 'Next scoreboard page' }).click();
  await expect(page.locator('.score-row')).toHaveCount(6);
  await assertGameViewport(page);
  await page.mouse.wheel(0, 1600);
  expect(await page.evaluate(() => scrollY)).toBe(0);
  await page.getByRole('button', { name: 'Leave room', exact: true }).click();
  await assertGameViewport(page);
});
