import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFile } from 'node:fs/promises';

test('@claim:private-join @claim:approved-phone host approves a phone through the private QR link', async ({ page, browser }) => {
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
  await page.getByRole('link', { name: /Create a real rehearsal room/ }).click();
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

test('routes have direct titles, a sitemap, and a designed HTTP 404 page', async ({ page }) => {
  await page.goto('/privacy');
  await expect(page).toHaveTitle('Privacy — Scene Cues');
  await expect(page.locator('main h1')).toHaveCount(1);
  await page.goto('/terms');
  await expect(page).toHaveTitle('Terms — Scene Cues');
  const sitemap = await page.request.get('/sitemap.xml');
  expect(sitemap.status()).toBe(200);
  expect(await sitemap.text()).toContain('/demo');
  const missing = await page.goto('/not-a-scene-cues-route');
  expect(missing?.status()).toBe(404);
  await expect(page).toHaveTitle('Page not found — Scene Cues');
  await expect(page.getByRole('heading', { name: 'This page is not available' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Return to Scene Cues' })).toBeVisible();
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

test('@claim:offline-reload offline reload keeps the demo sample available', async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('/demo');
  await page.evaluate(() => navigator.serviceWorker.ready.then(() => undefined));
  await expect.poll(() => page.evaluate(() => caches.keys())).toContain('scene-cues-shell-v3');
  await page.evaluate(() => navigator.serviceWorker.getRegistration().then((registration) => registration?.update()));
  await page.reload();
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByRole('heading', { name: /Run shared rehearsal cues/i })).toBeVisible();
  await context.close();
});

test('@claim:demo-sandbox sample rehearsal is populated, isolated, and resettable', async ({ page }) => {
  const requested: string[] = [];
  page.on('request', (request) => requested.push(request.url()));
  await page.goto('/demo');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByRole('heading', { name: /Run shared rehearsal cues/i })).toBeVisible();
  const scan = await new AxeBuilder({ page }).analyze();
  expect(scan.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact || ''))).toEqual([]);
  await expect(page.getByText('The Lantern Room', { exact: false })).toBeVisible();
  await expect(page.locator('.demo-log tbody tr')).toHaveCount(3);
  await page.getByRole('button', { name: 'Approve Alex — props' }).click();
  await page.getByRole('button', { name: 'Send GO for next cue' }).click();
  await expect(page.locator('.demo-log tbody tr')).toHaveCount(4);
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('.demo-log tbody tr')).toHaveCount(3);
  expect(requested.some((url) => new URL(url).pathname.startsWith('/api/rooms'))).toBe(false);
  const demoKeys = await page.evaluate(() => Object.keys(localStorage));
  expect(demoKeys.every((key) => key.startsWith('demo:'))).toBe(true);
});

test('@claim:demo-approval a pending sample phone cannot send GO until approved', async ({ page }) => {
  await page.goto('/demo');
  const go = page.getByRole('button', { name: 'Send GO for next cue' });
  await expect(go).toBeDisabled();
  await page.getByRole('button', { name: 'Approve Alex — props' }).click();
  await expect(go).toBeEnabled();
  await go.click();
  await expect(page.locator('.alert')).toContainText('Receipt 4');
  await expect(page.locator('.sample-state')).toContainText('Show rules camera');
});

test('@claim:cue-limit sample room stops adding cues after twelve', async ({ page }) => {
  await page.goto('/demo');
  const addCue = page.getByRole('button', { name: 'Add sample cue' });
  await addCue.click();
  await addCue.click();
  await expect(page.getByRole('heading', { name: '12 of 12 sample cues' })).toBeVisible();
  await addCue.click();
  await expect(page.locator('.alert')).toContainText('A room can hold up to 12 cues.');
});

test('@claim:csv-export sample receipts download as CSV', async ({ page }) => {
  await page.goto('/demo');
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download sample CSV' }).click();
  const file = await download;
  expect(file.suggestedFilename()).toBe('scene-cues-sample-log.csv');
  expect(await readFile((await file.path())!, 'utf8')).toContain('"Sequence","Time","Scene","Cue","Controller"');
});

test('@claim:log-deletion clear sample log removes every displayed receipt', async ({ page }) => {
  await page.goto('/demo');
  await expect(page.locator('.demo-log tbody tr')).toHaveCount(3);
  await page.getByRole('button', { name: 'Clear sample log' }).click();
  await expect(page.getByText('No sample receipts remain.')).toBeVisible();
  await expect(page.locator('.demo-log tbody tr')).toHaveCount(0);
});

test('@claim:keyboard-go approved sample controller can send GO with G', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Approve Alex — props' }).click();
  await page.locator('body').focus();
  await page.keyboard.press('g');
  await expect(page.locator('.alert')).toContainText('Receipt 4');
});

test('@claim:privacy-same-origin demo activity only contacts this site', async ({ page }) => {
  const requested: string[] = [];
  page.on('request', (request) => requested.push(request.url()));
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Approve Alex — props' }).click();
  await page.getByRole('button', { name: 'Send GO for next cue' }).click();
  const origin = new URL(page.url()).origin;
  expect(requested.every((url) => new URL(url).origin === origin)).toBe(true);
});
