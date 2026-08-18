<script setup lang="ts">
import type { ChallengeRes } from '~/composables/useWebauthn'

const auth = useAuth()
const notify = useNotify()
const { t } = useT()
await auth.loadMe()

const sessionRows = computed(() => [
  [t('cabinet-player'), auth.user.value?.username || 'Player'],
  ['Discord', auth.user.value?.discord_username || 'Unknown'],
  ['Minecraft UUID', auth.user.value?.uuid || 'Not loaded']
])

const hideFromOnline = ref((auth.user.value as any)?.hide_from_online || false)

async function toggleHideFromOnline() {
  try {
    await auth.request('/api/me/hide-from-online', {
      method: 'PUT',
      body: { hide: hideFromOnline.value }
    })
    notify.ok()
  } catch (e) {
    notify.fail(e)
    hideFromOnline.value = !hideFromOnline.value
  }
}

const canSilentJoin = computed(() =>
  auth.hasPermission('noro.mod.vanish.silent_join') || auth.hasPermission('noro.mod.vanish.use')
)
const silentJoin = ref((auth.user.value as any)?.silent_join || false)

async function toggleSilentJoin() {
  try {
    await auth.request('/api/me/silent-join', {
      method: 'PUT',
      body: { silent: silentJoin.value }
    })
    notify.ok()
  } catch (e) {
    notify.fail(e)
    silentJoin.value = !silentJoin.value
  }
}

const passkeys = ref<any[]>([])
const loadingPasskeys = ref(false)

async function loadPasskeys() {
  loadingPasskeys.value = true
  try {
    passkeys.value = await auth.request<any[]>('/api/me/passkeys')
  } catch (e) {
    console.error(e)
  } finally {
    loadingPasskeys.value = false
  }
}

async function addPasskey() {
  try {
    const opts = await auth.request<ChallengeRes>('/api/me/passkeys/register/options', { method: 'POST' })
    const credential = await createCredential(opts)

    await auth.request('/api/me/passkeys/register/verify', {
      method: 'POST',
      body: {
        state_id: opts.state_id,
        name: navigator.userAgent.includes('Mac') ? 'Touch ID / Mac Passkey' : 'Passkey',
        credential,
      }
    })
    notify.ok()
    await loadPasskeys()
  } catch (e) {
    notify.fail(e, 'Failed to register Passkey')
  }
}

async function removePasskey(id: string) {
  try {
    await auth.request(`/api/me/passkeys/${id}`, { method: 'DELETE' })
    notify.ok()
    await loadPasskeys()
  } catch (e) {
    notify.fail(e)
  }
}

onMounted(() => loadPasskeys())
</script>

<template>
  <NoroShell :title="t('nav-cabinet-settings')" subtitle="Your account">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-arrow-left" to="/cabinet">{{ t('nav-cabinet-home') }}</AtomButton>
    </template>

    <div class="grid gap-4 w-full">
      <!-- Account Info -->
      <section class="noro-panel p-6">
        <h2 class="noro-label mb-4">{{ t('cabinet-settings-account') }}</h2>
        <div class="grid gap-2 text-sm">
          <div
            v-for="[label, value] in sessionRows"
            :key="label"
            class="grid gap-1 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3 md:grid-cols-[160px_1fr] md:items-center"
          >
            <span class="noro-label">{{ label }}</span>
            <span class="break-all text-[var(--noro-text)]">{{ value }}</span>
          </div>
        </div>
      </section>

      <!-- Online Privacy & Vanish -->
      <section class="noro-panel p-6 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-eye-off" class="size-4 text-[var(--noro-blue)]" />
              {{ t('cabinet-privacy-title') }}
            </h2>
            <p class="text-xs text-[var(--noro-muted)]">{{ t('cabinet-privacy-hide-online') }}</p>
          </div>
          <input
            v-model="hideFromOnline"
            type="checkbox"
            class="size-5 accent-[var(--noro-magenta)] cursor-pointer"
            @change="toggleHideFromOnline"
          >
        </div>

        <div v-if="canSilentJoin" class="flex items-center justify-between border-t border-[var(--noro-border)] pt-4">
          <div>
            <h3 class="text-sm font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-ghost" class="size-4 text-[var(--noro-magenta)]" />
              {{ t('cabinet-privacy-silent-join-title') }}
            </h3>
            <p class="text-xs text-[var(--noro-muted)]">{{ t('cabinet-privacy-silent-join-hint') }}</p>
          </div>
          <input
            v-model="silentJoin"
            type="checkbox"
            class="size-5 accent-[var(--noro-magenta)] cursor-pointer"
            @change="toggleSilentJoin"
          >
        </div>
      </section>

      <!-- Passkeys (WebAuthn) -->
      <section class="noro-panel p-6 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-key-round" class="size-4 text-[var(--noro-blue)]" />
              {{ t('cabinet-passkeys-title') }}
            </h2>
            <p class="text-xs text-[var(--noro-muted)]">{{ t('cabinet-passkeys-lead') }}</p>
          </div>
          <AtomButton variant="secondary" icon="i-lucide-plus" @click="addPasskey">
            {{ t('cabinet-passkeys-add') }}
          </AtomButton>
        </div>

        <div v-if="passkeys.length" class="grid gap-2">
          <div
            v-for="pk in passkeys"
            :key="pk.id"
            class="flex items-center justify-between rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] p-3 text-xs"
          >
            <div class="flex items-center gap-3">
              <UIcon name="i-lucide-fingerprint" class="size-5 text-[var(--noro-blue)]" />
              <div>
                <div class="font-bold text-[var(--noro-text)]">{{ pk.name }}</div>
                <div class="text-[10px] text-[var(--noro-muted)]">
                  {{ t('cabinet-passkeys-created', { date: new Date(pk.created_at).toLocaleDateString() }) }}
                  ·
                  {{ pk.last_used_at ? t('cabinet-passkeys-used', { date: new Date(pk.last_used_at).toLocaleDateString() }) : t('cabinet-passkeys-unused') }}
                </div>
              </div>
            </div>
            <button
              class="rounded p-1 text-[var(--noro-muted)] hover:bg-red-500/20 hover:text-red-400 transition"
              :title="t('web-rules-scope-general')"
              @click="removePasskey(pk.id)"
            >
              <UIcon name="i-lucide-trash-2" class="size-4" />
            </button>
          </div>
        </div>

        <EmptyState
          v-else
          icon="i-lucide-shield-off"
          :title="t('cabinet-passkeys-none-title')"
          :text="t('cabinet-passkeys-none-text')"
        />
      </section>

      <!-- Sign out -->
      <section class="noro-panel p-6">
        <h2 class="noro-label mb-2">{{ t('cabinet-settings-session') }}</h2>
        <p class="mb-4 text-sm text-[var(--noro-muted)]">
          {{ t('cabinet-settings-signout-hint') }}
        </p>
        <AtomButton variant="secondary" icon="i-lucide-log-out" @click="auth.signOut()">{{ t('profile-sign-out') }}</AtomButton>
      </section>
    </div>
  </NoroShell>
</template>
