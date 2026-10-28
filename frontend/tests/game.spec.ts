import { test, expect, type Page } from '@playwright/test';

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
  await page.evaluate(() => {
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    window.scrollTo(0, 0);
  });
  await page.screenshot({ path: `test-results/${name}.png`, fullPage: true });
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
  await expect(page.getByRole('heading', { name: /Your best shot/ })).toBeVisible();
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
    await readyAndAdvance(page);
  }
  await expect(page.getByRole('heading', { name: 'The final tally' })).toBeVisible();
  expect(errors).toEqual([]);
  await capture(page, 'results');
});

test('two peers share readiness, category edits reset agreement, and refresh resumes identity', async ({ page, context }) => {
  await page.goto('/');
  await page.getByLabel('Your name').fill('Ada');
  await page.getByLabel('Time to explore').selectOption('60');
  await page.getByRole('button', { name: 'Let’s make a room' }).click();
  await expect(page.getByRole('heading', { name: 'The curious crew' })).toBeVisible();
  const second = await context.newPage();
  await second.goto(page.url());
  await second.getByLabel('Your name').fill('Linus');
  await second.getByRole('button', { name: 'Join the adventure' }).click();
  await expect(page.getByText('Linus', { exact: true })).toBeVisible();
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
  await expect(page.getByRole('heading', { name: /Your best shot/ })).toBeVisible();
  await expect(second.getByRole('heading', { name: /Your best shot/ })).toBeVisible();
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
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).not.toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await capture(page, 'home');
});
