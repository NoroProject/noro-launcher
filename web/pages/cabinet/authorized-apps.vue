<script setup lang="ts">
const auth = useAuth()
const notify = useNotify()

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
    // Пустой список выглядел как «доступа ни у кого нет» — ровно то, что игрок
    // хочет увидеть, и ровно то, чего мы не проверяли.
    apps.value = []
    notify.fail(e, 'Could not load authorized apps')
  } finally {
    loading.value = false
  }
}

async function revokeApp(id: string, name: string) {
  if (!confirm(`Отозвать доступ для приложения "${name}"?`)) return
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
  <NoroShell title="AUTHORIZED APPS" subtitle="Управление приложениями с доступом к аккаунту">
    <div class="grid gap-4">
      <section class="noro-panel p-6 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-base font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-shield-check" class="size-4 text-[var(--noro-blue)]" />
              Подключённые приложения
            </h2>
            <p class="text-xs text-[var(--noro-muted)]">Сторонние сервисы и лаунчеры, у которых есть доступ к вашему профилю</p>
          </div>
          <AtomButton variant="secondary" icon="i-lucide-rotate-cw" equal @click="loadApps">
            Обновить
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
                <div class="text-[11px] text-[var(--noro-muted)]">{{ app.description || 'Доступ к вашему профилю Noro Network' }}</div>
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
              <span>Отозвать доступ</span>
            </button>
          </div>
        </div>

        <EmptyState
          v-else
          icon="i-lucide-shield-off"
          title="У вас нет подключённых сторонних приложений"
          text="Здесь будут отображаться приложения и лаунчеры, которым вы разрешили доступ"
        />
      </section>
    </div>
  </NoroShell>
</template>
