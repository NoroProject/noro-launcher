<script setup lang="ts">
/**
 * Приложения, которым игрок открыл доступ к аккаунту.
 *
 * Показываем не только название, но и что именно приложение получило: список
 * выданных доступов — единственное, по чему можно решить, оставлять ли его.
 */

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

interface AuthorizedApp {
  id: string
  app_id: string
  name: string
  description: string | null
  icon_url: string | null
  authorized_at: string
  scopes: { name: string; title: string; tier: string }[]
}

const apps = ref<AuthorizedApp[]>([])
const loading = ref(true)
const revoking = ref<string | null>(null)

const scopeText = (s: { name: string; title: string }) => {
  const key = `oauth-scope-${s.name.replace(':', '-')}`
  const translated = t(key)
  return translated === key ? s.title : translated
}

async function load() {
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

async function revoke(app: AuthorizedApp) {
  if (!confirm(t('cabinet-apps-revoke-confirm', { name: app.name }))) return
  revoking.value = app.app_id
  try {
    await auth.request(`/api/me/authorized-apps/${app.app_id}`, { method: 'DELETE' })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    revoking.value = null
  }
}

onMounted(load)
</script>

<template>
  <NoroCard :title="t('cabinet-apps-connected')" :subtitle="t('cabinet-apps-lead')" icon="i-lucide-shield-check">
    <template #actions>
      <AtomButton variant="secondary" icon="i-lucide-rotate-cw" size="sm" @click="load" />
    </template>

    <div v-if="loading" class="flex justify-center py-10">
      <UIcon name="i-lucide-loader-2" class="size-6 animate-spin text-[var(--noro-blue)]" />
    </div>

    <div v-else-if="apps.length" class="grid gap-2">
      <div
        v-for="app in apps"
        :key="app.id"
        class="rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4"
      >
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="flex min-w-0 items-center gap-3">
            <div class="grid size-10 shrink-0 place-items-center overflow-hidden rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)]">
              <img v-if="app.icon_url" :src="app.icon_url" alt="" class="size-full object-cover">
              <UIcon v-else name="i-lucide-app-window" class="size-5 text-[var(--noro-blue)]" />
            </div>
            <div class="min-w-0">
              <div class="truncate text-sm font-bold text-[var(--noro-text)]">{{ app.name }}</div>
              <div class="truncate text-[11px] text-[var(--noro-muted)]">
                {{ t('cabinet-apps-since', { date: new Date(app.authorized_at).toLocaleDateString() }) }}
              </div>
            </div>
          </div>

          <AtomButton
            variant="danger"
            size="sm"
            icon="i-lucide-trash-2"
            :loading="revoking === app.app_id"
            @click="revoke(app)"
          >
            {{ t('cabinet-apps-revoke') }}
          </AtomButton>
        </div>

        <ul v-if="app.scopes?.length" class="mt-3 flex flex-wrap gap-1.5">
          <li
            v-for="s in app.scopes"
            :key="s.name"
            class="rounded bg-[var(--noro-panel-2)] px-2 py-1 text-[11px] text-[var(--noro-muted)]"
          >
            {{ scopeText(s) }}
          </li>
        </ul>
      </div>
    </div>

    <EmptyState
      v-else
      icon="i-lucide-shield-off"
      :title="t('cabinet-apps-none-title')"
      :text="t('cabinet-apps-none-text')"
    />
  </NoroCard>
</template>
