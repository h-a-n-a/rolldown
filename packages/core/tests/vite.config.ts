import { defineConfig } from 'vitest/config'
export default defineConfig({
  resolve: {
    alias: {
      '@': __dirname,
    },
  },
  test: {
    isolate: false,
    threads: false,
    include: ['./function/index.test.ts'],
  },
})
