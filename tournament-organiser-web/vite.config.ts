import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import VueI18nPlugin from '@intlify/unplugin-vue-i18n/vite'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false,
  },
  plugins: [
    vue(),
    VueI18nPlugin({
      include: [path.resolve(__dirname, './src/locales/**')],
    }),
  ],
  // NOTE: apparently, localhost may not resolve well with http-node-proxy, at
  // least on my machine, which is why you may need 127.0.0.1 for development.
  //
  // During development, you want to spin up a vite server with only the
  // frontend, so you get hot-reload. Otherwise, you will need to restart the
  // API server, and it takes 5s+, which is really annoying after a while.
  //
  // In production, no change of origin should happen.
  //
  // There is no modification of url needed. In development, when using
  // `npm run dev`, all /api url gets their origin changed. This does not affect
  // production because no vite-server is running.
  //
  // This configuration allows to bypass all CORS issue by changing the origin
  // so fetch API can still be used without needing axios. Fetch API will make
  // the body of a POST response unreadable (opaque) for security reasons.
  // see https://stackoverflow.com/a/54906434
  server: {
    proxy: {
      '/api': {
        // https://vitejs.dev/config/server-options#server-proxy
        // 500 error, maybe my pc is an issue, but it looks like an oversight of
        // localhost not being rewritten to 127.0.0.1
        // target: 'http://localhost:8080',
        // 404
        // target: 'http://jsonplaceholder.typicode.com',
        // works without problem
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      /*       'vue-i18n': 'vue-i18n/dist/vue-i18n.cjs.js',
       */
    },
  },
})
