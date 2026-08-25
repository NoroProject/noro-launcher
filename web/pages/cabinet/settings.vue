<script setup lang="ts">
import type { ChallengeRes } from '~/composables/useWebauthn'

const auth = useAuth()
const notify = useNotify()
const { t } = useT()
await auth.loadMe()

const sessionRows = computed(() => [
  [t('cabinet-player'), auth.user.value?.username || 'Player'],
  ['Minecraft UUID', auth.user.value?.uuid || 'Not loaded']
])

const methods = ref<AuthMethods>({ providers: [], passkey: false })

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

onMounted(async () => {
  methods.value = await loadAuthMethods()
  // Ключей нет как способа входа — и карточки их быть не должно.
  if (methods.value.passkey) await loadPasskeys()
})
</script>

<template>
  <NoroShell :title="t('nav-cabinet-settings')" subtitle="Your account">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-arrow-left" :to="link.cabinet()">{{ t('nav-cabinet-home') }}</AtomButton>
    </template>

    <div class="grid gap-4 w-full">
      <!-- Account Info -->
      <NoroCard :title="t('cabinet-settings-account')" icon="i-lucide-id-card">
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
      </NoroCard>

      <CabinetLinkedPlatforms :providers="methods.providers" />

      <!-- Online Privacy & Vanish -->
      <NoroCard
        :title="t('cabinet-privacy-title')"
        :subtitle="t('cabinet-privacy-hide-online')"
        icon="i-lucide-eye-off"
      >
        <template #actions>
          <input
            v-model="hideFromOnline"
            type="checkbox"
            class="size-5 accent-[var(--noro-magenta)] cursor-pointer"
            @change="toggleHideFromOnline"
          >
        </template>

        <div v-if="canSilentJoin" class="flex items-center justify-between border-t border-[var(--noro-border)] pt-4 -mt-1">
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
      </NoroCard>

      <!-- Passkeys (WebAuthn). Способ выключен оператором — карточки нет. -->
      <NoroCard
        v-if="methods.passkey"
        :title="t('cabinet-passkeys-title')"
        :subtitle="t('cabinet-passkeys-lead')"
        icon="i-lucide-key-round"
      >
        <template #actions>
          <AtomButton variant="secondary" icon="i-lucide-plus" @click="addPasskey">
            {{ t('cabinet-passkeys-add') }}
          </AtomButton>
        </template>

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
      </NoroCard>

      <!-- Sign out -->
      <NoroCard
        :title="t('cabinet-settings-session')"
        :subtitle="t('cabinet-settings-signout-hint')"
        icon="i-lucide-log-out"
      >
        <AtomButton variant="danger" icon="i-lucide-log-out" @click="auth.signOut()">{{ t('profile-sign-out') }}</AtomButton>
      </NoroCard>
    </div>
  </NoroShell>
</template>
