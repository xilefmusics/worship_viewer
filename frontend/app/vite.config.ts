import { execSync } from 'node:child_process'
import path from 'node:path'

import tailwindcss from '@tailwindcss/vite'
import { tanstackRouter } from '@tanstack/router-plugin/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, loadEnv, type Plugin } from 'vite'
import { VitePWA } from 'vite-plugin-pwa'

import { AV_OUTPUT_CSP, isAvOutputPath } from './src/lib/player/av-output-csp'

function gitVersion(): string {
  try {
    return execSync('git describe --tags --always --dirty', {
      stdio: ['ignore', 'pipe', 'ignore'],
    })
      .toString()
      .trim()
  } catch {
    return '0.0.0-dev'
  }
}

function avOutputCspPlugin(): Plugin {
  const apply = (
    req: { url?: string },
    res: { setHeader: (name: string, value: string) => void },
    next: () => void,
  ) => {
    if (isAvOutputPath(req.url ?? '')) {
      res.setHeader('Content-Security-Policy', AV_OUTPUT_CSP)
    }
    next()
  }
  return {
    name: 'av-output-csp',
    configureServer(server) {
      server.middlewares.use(apply)
    },
    configurePreviewServer(server) {
      server.middlewares.use(apply)
    },
  }
}

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const proxyTarget = env.VITE_DEV_PROXY_TARGET || 'http://127.0.0.1:8080'

  return {
    plugins: [
      avOutputCspPlugin(),
      tanstackRouter({ target: 'react', autoCodeSplitting: true }),
      react(),
      tailwindcss(),
      VitePWA({
        registerType: 'prompt',
        includeAssets: ['favicon.png', 'apple-touch-icon.png', 'brand/**/*.png', 'fonts/*.woff2'],
        manifest: {
          name: 'Worship Viewer',
          short_name: 'Worship',
          description: 'Worship library for your team',
          theme_color: '#d01d21',
          background_color: '#ffffff',
          display: 'standalone',
          display_override: ['window-controls-overlay', 'standalone'],
          start_url: '/',
          scope: '/',
          icons: [
            {
              src: '/brand/icon-192.png',
              sizes: '192x192',
              type: 'image/png',
            },
            {
              src: '/brand/icon-512.png',
              sizes: '512x512',
              type: 'image/png',
            },
            {
              src: '/brand/icon-maskable-512.png',
              sizes: '512x512',
              type: 'image/png',
              purpose: 'maskable',
            },
          ],
        },
        workbox: {
          navigateFallback: 'index.html',
          navigateFallbackDenylist: [/^\/api\//, /^\/auth\//, /^\/player\/output/],
        },
      }),
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, 'src'),
      },
    },
    optimizeDeps: {
      exclude: ['@worshipviewer/chordlib-wasm'],
    },
    define: {
      __APP_VERSION__: JSON.stringify(gitVersion()),
      __APP_BUILD_DATE__: JSON.stringify(new Date().toISOString()),
    },
    server: {
      proxy: {
        '/api': { target: proxyTarget, changeOrigin: true, ws: true },
        '/auth': { target: proxyTarget, changeOrigin: true },
      },
    },
    build: {
      // Pin Safari 16 support instead of inheriting Vite's moving baseline target.
      target: ['safari16', 'es2020'],
      rollupOptions: {
        output: {
          manualChunks(id) {
            if (id.includes('node_modules/react-dom') || id.includes('node_modules/react/')) {
              return 'vendor'
            }
            if (
              id.includes('node_modules/@tanstack/react-query') ||
              id.includes('node_modules/@tanstack/react-router')
            ) {
              return 'query'
            }
            if (id.includes('node_modules/@codemirror/') || id.includes('node_modules/@uiw/react-codemirror')) {
              return 'codemirror'
            }
            if (id.includes('node_modules/motion')) {
              return 'motion'
            }
          },
        },
      },
    },
  }
})
