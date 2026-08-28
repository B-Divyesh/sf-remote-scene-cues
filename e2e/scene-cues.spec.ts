import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('host approves a phone and receives an ordered cue', async ({ page, browser }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  await page.goto('/');
  await expect(page.locator('h1')).toHaveCount(1);
  const scan = await new AxeBuilder({ page }).analyze();
  expect(scan.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact || ''))).toEqual([]);

  await page.locator('#show-name').fill('E2E dress rehearsal');
  await page.locator('#cues').fill('Opening | House lights\nOpening | Start music');
  await page.getByRole('button', { name: 'Create room + private QR' }).click();
  await expect(page).toHaveURL(/\/host\/[A-Z0-9]{6}$/);
  await expect(page.locator('h1')).toHaveText('E2E dress rehearsal');
  const hostScan = await new AxeBuilder({ page }).analyze();
  expect(hostScan.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact || ''))).toEqual([]);
  const joinUrl = await page.locator('.join-link').getAttribute('href');
  expect(joinUrl).toBeTruthy();
  expect(new URL(joinUrl!).origin).toBe(new URL(page.url()).origin);

  const phoneContext = await browser.newContext({
    viewport: { width: 390, height: 844 },
    extraHTTPHeaders: { 'x-forwarded-for': '192.0.2.20' }
  });
  const phone = await phoneContext.newPage();
  const phoneConsoleErrors: string[] = [];
  phone.on('console', (message) => { if (message.type() === 'error') phoneConsoleErrors.push(message.text()); });
  await phone.goto(joinUrl!);
  await phone.locator('#controller-name').fill('Sam — booth');
  await phone.getByRole('button', { name: 'Request host approval' }).click();
  await expect(phone.getByText('Approval pending')).toBeVisible();

  await expect(page.getByText('Sam — booth')).toBeVisible();
  await page.getByRole('button', { name: 'Approve' }).click();
  await expect(phone.getByRole('button', { name: /GO/ })).toBeVisible();
  const controllerScan = await new AxeBuilder({ page: phone }).analyze();
  expect(controllerScan.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact || ''))).toEqual([]);
  await phone.getByRole('button', { name: /GO/ }).click();
  await expect(phone.locator('.receipt')).toContainText('Receipt 1');
  await expect(page.locator('.cue-list .current')).toContainText('House lights');

  expect(consoleErrors).toEqual([]);
  expect(phoneConsoleErrors).toEqual([]);
  await phoneContext.close();
});

test('mobile landing page preserves a single heading and reachable setup', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page.getByRole('link', { name: /Buy Cue Book/i })).toHaveCount(0);
  await expect(page.locator('a[href*="api.sociobot.in"]')).toHaveCount(0);
  await expect(page.locator('.label-row')).toContainText('/ 12');
  await page.getByRole('link', { name: /Open a rehearsal room/ }).click();
  await expect(page.locator('#show-name')).toBeInViewport();
});

test('navigation and legal links meet the 44px target contract', async ({ page }) => {
  await page.goto('/');
  const targets = page.locator('.wordmark, footer nav a');
  await expect(targets).toHaveCount(4);
  for (let index = 0; index < await targets.count(); index += 1) {
    const box = await targets.nth(index).boundingBox();
    expect(box?.height).toBeGreaterThanOrEqual(44);
  }
});

test('oversize request recovery copy states the authoritative 12 cue limit', async ({ page }) => {
  await page.route('**/api/rooms', (route) => route.fulfill({ status: 413, body: 'body too large' }));
  await page.goto('/');
  await page.locator('#show-name').fill('Oversize rehearsal');
  await page.locator('#cues').fill('Opening | House lights');
  await page.getByRole('button', { name: 'Create room + private QR' }).click();
  await expect(page.getByRole('alert')).toContainText(
    'That cue sheet is too large. Keep it to 12 short cues and shorten long cue text.'
  );
});

test('host cue list supports documented arrow-key selection', async ({ page }) => {
  await page.goto('/');
  await page.locator('#show-name').fill('Keyboard rehearsal');
  await page.locator('#cues').fill('Opening | House lights\nOpening | Start music\nFinale | Full lights');
  await page.getByRole('button', { name: 'Create room + private QR' }).click();
  const cues = page.locator('.cue-list button');
  await cues.nth(0).focus();
  await page.keyboard.press('ArrowDown');
  await expect(cues.nth(1)).toBeFocused();
  await page.keyboard.press('ArrowUp');
  await expect(cues.nth(0)).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('.cue-list .current')).toContainText('House lights');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Space');
  await expect(page.locator('.cue-list .current')).toContainText('Start music');
  await expect(page.locator('.go-compact')).toBeEnabled();
  await page.keyboard.press('g');
  await expect(page.locator('.cue-list .current')).toContainText('Full lights');
});

test('offline reload is served by the versioned application shell', async ({ page, context }) => {
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready.then(() => undefined));
  await expect.poll(() => page.evaluate(() => caches.keys())).toContain('scene-cues-shell-v2');
  await page.evaluate(() => navigator.serviceWorker.getRegistration().then((registration) => registration?.update()));
  await page.reload();
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByRole('heading', { name: /Advance the scene/i })).toBeVisible();
  await context.setOffline(false);
});
