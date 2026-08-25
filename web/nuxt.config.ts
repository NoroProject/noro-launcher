export default defineNuxtConfig({
  compatibilityDate: '2026-06-04',
  devtools: { enabled: true },
  experimental: {
    appManifest: false
  },
  modules: ['@nuxt/ui', '@sentry/nuxt/module'],
  // Сборка ничего никуда не отправляет: ни исходные карты, ни телеметрию
  // плагина. Отчёты об ошибках идут только из рантайма и только при заданном DSN.
  sentry: {
    sourceMapsUploadOptions: { enabled: false, telemetry: false }
  },
  // Интерфейс тёмный по замыслу, светлой версии у него нет. Без этого Nuxt UI
  // слушал системную тему браузера, и у половины людей его собственные окна —
  // модалки, поповеры — приезжали белыми поверх тёмной страницы.
  colorMode: {
    preference: 'dark',
    fallback: 'dark',
    // Ключ хранения сменён намеренно: в браузерах уже лежит `system` с тех
    // пор, когда тема шла за настройкой ОС, и сохранённое значение сильнее
    // `preference` — со старым ключом светлая тема возвращалась бы вечно.
    storageKey: 'noro-theme',
  },
  css: ['~/assets/css/main.css'],
  // Логотипы платформ входа. Свои SVG, а не коллекция бренд-иконок с npm:
  // их ровно три, и тянуть ради них ещё один пакет и его серверный бандл
  // незачем. Иконки одноцветные — цвет задаёт вёрстка через `currentColor`.
  icon: {
    customCollections: [{ prefix: 'brand', dir: './assets/icons' }]
  },
  // No fallback domain here on purpose: a build without these variables used to
  // point at one specific production deployment, and nothing said so.
  // `useApi` reports the missing variable instead.
  runtimeConfig: {
    public: {
      masterUrl: process.env.NUXT_PUBLIC_MASTER_URL || '',
      webUrl: process.env.NUXT_PUBLIC_WEB_URL || '',
      // Empty DSN keeps Sentry inert — see sentry.client.config.ts.
      sentryDsn: process.env.NUXT_PUBLIC_SENTRY_DSN || '',
      sentryEnvironment: process.env.NUXT_PUBLIC_SENTRY_ENVIRONMENT || ''
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
