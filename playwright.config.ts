import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  use: {
    baseURL: 'http://localhost:1420',
    browserName: 'chromium',
    channel: process.env.CI ? undefined : 'chrome',
    headless: true,
  },
  webServer: { command: 'pnpm dev', url: 'http://localhost:1420', reuseExistingServer: false },
  reporter: 'list',
  outputDir: '.local/playwright',
});
