import * as Sentry from '@sentry/nuxt'

/**
 * Browser-side error reporting.
 *
 * No DSN means no reporting — the module stays inert instead of guessing an
 * endpoint. A build that reports nowhere is a configuration choice; a build
 * that reports somewhere unexpected is a leak.
 */
const dsn = useRuntimeConfig().public.sentryDsn

if (dsn) {
  Sentry.init({
    dsn,
    environment: useRuntimeConfig().public.sentryEnvironment || 'development',
    // Errors only. Performance tracing would spend the event budget on
    // page loads and tell us nothing about the failures we are chasing.
    tracesSampleRate: 0
  })
}
