<script setup lang="ts">
const route = useRoute()
const auth = useAuth()
const notify = useNotify()

const busy = ref(false)
const message = ref<string | null>(null)
const showRecovery = ref(false)
const recoveryUser = ref('')
const recoveryCode = ref('')

const next = computed(() => String(route.query.next || '/cabinet'))
const loginHref = computed(() => auth.discordLoginUrl(`/login?next=${encodeURIComponent(next.value)}`))

async function loginWithPasskey() {
  busy.value = true
  message.value = null
  try {
    const opts = await auth.request<ChallengeRes>('/auth/passkeys/login/options', { method: 'POST' })
    const credential = await getCredential(opts)

    const res = await auth.request<any>('/auth/passkeys/login/verify', {
      method: 'POST',
      body: { state_id: opts.state_id, credential }
    })
    auth.token.value = res.access_token
    auth.user.value = res.user
    await navigateTo(next.value)
  } catch (e) {
    message.value = humanError(e)
  } finally {
    busy.value = false
  }
}

async function loginWithRecovery() {
  busy.value = true
  message.value = null
  try {
    const res = await auth.request<any>('/api/auth/recovery/login', {
      method: 'POST',
      body: { username: recoveryUser.value.trim(), code: recoveryCode.value.trim() },
    })
    auth.token.value = res.access_token
    auth.user.value = res.user
    // Код сгорел — следующий вход должен опираться на что-то долговечнее.
    await navigateTo(res.bind_passkey ? '/cabinet?bind_passkey=1' : next.value)
  } catch (e) {
    message.value = humanError(e)
  } finally {
    busy.value = false
  }
}

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
            <img src="/icon.png" />
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

        <div class="mt-8 space-y-3">
          <NuxtLink :to="loginHref" class="noro-cta block w-full px-6 py-4 text-center font-bold">
            {{ busy || auth.loading.value ? 'WAITING...' : 'SIGN IN WITH DISCORD' }}
          </NuxtLink>

          <button
            type="button"
            class="flex w-full items-center justify-center gap-2 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] py-3 text-xs font-bold text-[var(--noro-text)] hover:border-[var(--noro-blue)] hover:text-[var(--noro-blue)] transition"
            :disabled="busy"
            @click="loginWithPasskey"
          >
            <UIcon name="i-lucide-key-round" class="size-4 text-[var(--noro-blue)]" />
            SIGN IN WITH PASSKEY
          </button>

          <!-- Не «аварийный вариант»: passkey нельзя привязать по http:// без
               домена, и до настройки TLS код — единственный путь внутрь. -->
          <button
            v-if="!showRecovery"
            type="button"
            class="w-full py-2 text-[11px] font-bold uppercase tracking-wider text-[var(--noro-muted)] hover:text-[var(--noro-cream)] transition"
            @click="showRecovery = true"
          >
            Use a recovery code
          </button>

          <form v-else class="grid gap-2 rounded-lg border border-[var(--noro-border)] p-4" @submit.prevent="loginWithRecovery">
            <input v-model="recoveryUser" class="noro-input w-full" placeholder="Username" autocomplete="username">
            <input v-model="recoveryCode" class="noro-input w-full font-mono" placeholder="XXXX-XXXX-XXXX">
            <button
              type="submit"
              class="noro-cta w-full px-6 py-3 text-center text-xs font-bold"
              :disabled="busy || !recoveryUser.trim() || !recoveryCode.trim()"
            >
              SIGN IN
            </button>
            <p class="text-[10px] text-[var(--noro-muted)]">
              Each code works once. After signing in, bind a passkey — that is what the next login should rest on.
            </p>
          </form>
        </div>
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
