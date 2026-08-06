<script setup lang="ts">
const route = useRoute()
const auth = useAuth()
const busy = ref(false)
const message = ref<string | null>(null)

const next = computed(() => String(route.query.next || '/cabinet'))
const loginHref = computed(() => auth.discordLoginUrl(`/login?next=${encodeURIComponent(next.value)}`))

onMounted(async () => {
  const access = route.query.access_token
  const refresh = route.query.refresh_token
  if (typeof access !== 'string') {
    if (auth.token.value) await auth.loadMe()
    return
  }

  busy.value = true
  auth.token.value = access
  auth.refreshToken.value = typeof refresh === 'string' ? refresh : null
  const user = await auth.loadMe()
  busy.value = false
  if (user) {
    await navigateTo(next.value)
  } else {
    message.value = auth.error.value || 'Token received, but profile failed to load'
  }
})
</script>

<template>
  <div class="grid min-h-screen place-items-center px-4">
    <div class="noro-panel grid w-full max-w-5xl overflow-hidden p-0 md:grid-cols-[420px_1fr]">
      <section class="p-8 md:p-12">
        <div class="mb-12 flex items-center gap-3">
          <div class="grid size-12 place-items-center rounded-lg">
              <img src="/icon.png"/>
          </div>
          <div class="text-xs font-black uppercase tracking-wider text-[var(--noro-muted)]">Secure login</div>
        </div>

        <div>
          <h1 class="noro-pixel text-4xl leading-none text-[var(--noro-cream)]">NORO LAUNCHER</h1>
        </div>

        <UAlert
          v-if="message || auth.error.value"
          color="error"
          variant="subtle"
          icon="i-lucide-circle-alert"
          :description="message || auth.error.value || undefined"
          class="my-5"
        />

        <NuxtLink :to="loginHref" class="noro-cta mt-8 w-full px-6 py-4 text-center">
          {{ busy || auth.loading.value ? 'WAITING...' : 'SIGN IN WITH DISCORD' }}
        </NuxtLink>

      </section>

      <section class="relative hidden min-h-[520px] overflow-hidden bg-[var(--noro-bg-deep)] md:block">
        <div class="absolute left-28 top-28 size-72 rotate-45 border-[20px] border-[var(--noro-blue)]/35" />
        <div class="absolute left-40 top-40 size-48 rotate-12 bg-[var(--noro-magenta)]" />
        <div class="absolute right-24 top-24 size-20 rotate-12 bg-[var(--noro-cream)]" />
        <div class="absolute bottom-28 left-32 size-16 rotate-45 bg-[var(--noro-blue)]" />
        <div class="absolute bottom-12 right-12 h-20 w-40 bg-[var(--noro-panel-2)]" />
      </section>
    </div>
  </div>
</template>
