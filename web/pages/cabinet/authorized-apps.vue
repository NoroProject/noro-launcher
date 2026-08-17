<script setup lang="ts">
const auth = useAuth()
const notify = useNotify()
const { t } = useT()

await auth.loadMe()

interface AuthorizedApp {
  id: string
  name: string
  description: string | null
  icon_url: string | null
  authorized_at: string
  scopes: string
}

const apps = ref<AuthorizedApp[]>([])
const loading = ref(true)
const revokingId = ref<string | null>(null)

async function loadApps() {
  loading.value = true
  try {
    apps.value = await auth.request<AuthorizedApp[]>('/api/me/authorized-apps')
  } catch (e) {
    apps.value = []
    notify.fail(e, 'Could not load authorized apps')
  } finally {
    loading.value = false
  }
}

async function revokeApp(id: string, name: string) {
  if (!confirm(`Revoke access for "${name}"?`)) return
  revokingId.value = id
  try {
    await auth.request(`/api/me/authorized-apps/${id}`, { method: 'DELETE' })
    notify.ok()
    await loadApps()
  } catch (e) {
    notify.fail(e)
  } finally {
    revokingId.value = null
  }
}

onMounted(() => {
  loadApps()
})
</script>

<template>
  <NoroShell title="AUTHORIZED APPS" :subtitle="t('cabinet-apps-subtitle')">
    <div class="grid gap-4">
      <section class="noro-panel p-6 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-shield-check" class="size-4 text-[var(--noro-blue)]" />
              {{ t('cabinet-apps-title') }}
            </h2>
            <p class="text-xs text-[var(--noro-muted)]">{{ t('cabinet-apps-lead') }}</p>
          </div>
          <AtomButton variant="secondary" icon="i-lucide-rotate-cw" equal @click="loadApps">
            {{ t('cabinet-apps-refresh') }}
          </AtomButton>
        </div>

        <div v-if="loading" class="flex items-center justify-center py-12">
          <UIcon name="i-lucide-loader-2" class="size-6 animate-spin text-[var(--noro-blue)]" />
        </div>

        <div v-else-if="apps.length" class="grid gap-2">
          <div
            v-for="app in apps"
            :key="app.id"
            class="flex items-center justify-between rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4 text-xs"
          >
            <div class="flex items-center gap-3">
              <div class="grid size-10 place-items-center rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)]">
                <img v-if="app.icon_url" :src="app.icon_url" class="size-full rounded-lg object-cover" />
                <UIcon v-else name="i-lucide-app-window" class="size-5 text-[var(--noro-blue)]" />
              </div>
              <div>
                <div class="font-bold text-[var(--noro-text)] text-sm">{{ app.name }}</div>
                <div class="text-[11px] text-[var(--noro-muted)]">{{ app.description || t('cabinet-apps-default-desc') }}</div>
              </div>
            </div>

            <button
              type="button"
              class="flex items-center gap-1.5 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs font-bold text-red-400 transition hover:bg-red-500/20 disabled:opacity-50"
              :disabled="revokingId === app.id"
              @click="revokeApp(app.id, app.name)"
            >
              <UIcon v-if="revokingId === app.id" name="i-lucide-loader-2" class="size-3.5 animate-spin" />
              <UIcon v-else name="i-lucide-trash-2" class="size-3.5" />
              <span>{{ t('cabinet-apps-revoke') }}</span>
            </button>
          </div>
        </div>

        <EmptyState
          v-else
          icon="i-lucide-shield-off"
          :title="t('cabinet-apps-none-title')"
          :text="t('cabinet-apps-none-text')"
        />
      </section>
    </div>
  </NoroShell>
</template>
