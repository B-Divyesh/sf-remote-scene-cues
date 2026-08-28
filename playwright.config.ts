import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  retries: 0,
  workers: 1,
  reporter: 'line',
  use: {
    baseURL: 'http://127.0.0.1:8080',
    trace: 'retain-on-failure'
  },
  projects: [
    { name: 'desktop-chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile-chromium', use: { ...devices['iPhone 13'], browserName: 'chromium', viewport: { width: 390, height: 844 } } }
  ],
  webServer: {
    command: 'npm run build && DATABASE_URL="sqlite:///tmp/scene-cues-e2e.db?mode=rwc" PUBLIC_URL="http://127.0.0.1:8080" cargo run',
    url: 'http://127.0.0.1:8080/health',
    reuseExistingServer: false,
    timeout: 120_000
  }
});
