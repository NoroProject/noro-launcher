import * as Sentry from '@sentry/nuxt'

/** Nitro-side reporting. Same rule as the client: no DSN, no reporting. */
const dsn = process.env.NUXT_PUBLIC_SENTRY_DSN

if (dsn) {
  Sentry.init({
    dsn,
    environment: process.env.NUXT_PUBLIC_SENTRY_ENVIRONMENT || 'development',
    tracesSampleRate: 0
  })
}
