export default defineNuxtConfig({
  compatibilityDate: '2026-06-04',
  devtools: { enabled: true },
  experimental: {
    appManifest: false
  },
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  runtimeConfig: {
    public: {
      masterUrl: process.env.NUXT_PUBLIC_MASTER_URL || 'http://localhost:8080',
      webUrl: process.env.NUXT_PUBLIC_WEB_URL || 'http://localhost:3000'
    }
  },
  app: {
    head: {
      title: 'Noro Launcher',
      htmlAttrs: { lang: 'en' },
      meta: [
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        { name: 'description', content: 'Cabinet and admin panel for Noro Launcher' }
      ],
      // Шрифты локальные (public/fonts) — те же, что в лаунчере.
      link: [
        { rel: 'icon', type: 'image/png', href: '/favicon.png' }
      ]
    }
  }
})
