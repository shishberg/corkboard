import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests/parity',
  timeout: 60_000,
  // Run each spec file serially — parity spec spawns a device server per-file
  workers: 1,
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
