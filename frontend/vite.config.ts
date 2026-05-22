import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import { skillsPlugin } from './plugins/skillsPlugin'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), skillsPlugin()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    proxy: {
      '/api/knowledge': {
        target: 'http://localhost:3001',
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/api\/knowledge/, ''),
      },
      '/api/solvers': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/api\/solvers/, ''),
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
  },
})
