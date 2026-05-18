import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

const chunkGroups = [
  ['vendor', ['vue', 'vue-router']],
  ['animation', ['gsap']],
  ['viz', ['d3']],
]

function chunkNameForModule(id) {
  const normalized = id.replace(/\\/g, '/')
  const group = chunkGroups.find(([, deps]) => deps.some(dep => normalized.includes(`/node_modules/${dep}/`)))
  return group && group[0]
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    rollupOptions: {
      output: {
        manualChunks: chunkNameForModule,
      },
    },
  },
  server: {
    port: 5173,
    open: true,
  },
})
