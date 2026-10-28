import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 90000,
  use: {
    baseURL: 'http://127.0.0.1:5173',
    viewport: { width: 1440, height: 1000 },
    launchOptions: process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH, args: ['--no-sandbox'] } : {},
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'pnpm dev --host 127.0.0.1',
    url: 'http://127.0.0.1:5173',
    reuseExistingServer: !process.env.CI,
  },
});
