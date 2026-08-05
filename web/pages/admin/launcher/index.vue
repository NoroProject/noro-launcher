<script setup lang="ts">
import type { LauncherVersionRow } from '~/types/api'

const auth = useAuth()

const notify = useNotify()
await auth.loadMe()

const { data: versions, refresh, pending, error } = await useAsyncData('admin-launcher-versions', () =>
  auth.request<LauncherVersionRow[]>('/api/admin/launcher/versions'), { default: () => [] }
)

const tag = ref('')
const latest = ref<Record<string, unknown> | null>(null)
const jobId = ref('')
const busy = ref<string | null>(null)
const showBuild = ref(false)
const showLog = ref(false)

async function githubLatest() {
  busy.value = 'github'
  try {
    latest.value = await auth.request<Record<string, unknown>>('/api/admin/launcher/github')
    if (typeof latest.value?.tag === 'string') tag.value = latest.value.tag
  } finally {
    busy.value = null
  }
}

async function buildLauncher() {
  if (!tag.value) return
  busy.value = 'build'
  try {
    const response = await auth.request<{ job_id: string }>('/api/admin/launcher/build', { method: 'POST', body: { tag: tag.value } })
    jobId.value = response.job_id
    showBuild.value = false
    showLog.value = true
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = null
  }
}

/** Вся версия разом: пять платформ в двух видах — по одной их не наклацаешь. */
async function deployAll(ids: string[]) {
  busy.value = 'deploy-all'
  try {
    // Последовательно: мастер на каждый деплой рассылает лаунчерам обновление,
    // и параллельный залп сделал бы порядок рассылки случайным.
    for (const id of ids) {
      await auth.request(`/api/admin/launcher/deploy/${id}`, { method: 'POST' })
    }
    await refresh()
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = null
  }
}

async function deploy(versionId: string) {
  busy.value = `deploy-${versionId}`
  try {
    await auth.request(`/api/admin/launcher/deploy/${versionId}`, { method: 'POST' })
    await refresh()
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = null
  }
}
</script>

<template>
  <NoroShell title="LAUNCHER" subtitle="Versions, GitHub tag builds, and deploy">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
      <AtomButton variant="primary" icon="i-lucide-hammer" @click="showBuild = true">Build tag</AtomButton>
      <AtomButton variant="secondary" icon="i-lucide-file-text" @click="showLog = true">Build log</AtomButton>
    </template>

    <UAlert v-if="error" class="mb-5" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="humanError(error)" />

    <section class="noro-panel overflow-hidden">
      <LauncherVersionsTable
        v-if="versions?.length"
        :versions="versions"
        :busy="busy"
        @deploy="deploy"
        @deploy-many="deployAll"
      />
      <EmptyState v-else icon="i-lucide-rocket" title="No versions yet" text="Build a launcher tag from the toolbar." />
    </section>

    <AtomModal v-model="showBuild" title="GITHUB BUILD" subtitle="Build a launcher release tag">
      <div class="grid gap-3">
        <AtomButton variant="secondary" :loading="busy === 'github'" icon="i-lucide-github" @click="githubLatest">Check latest release</AtomButton>
        <pre v-if="latest" class="rounded-lg bg-[var(--noro-input)] p-3 text-xs text-[var(--noro-text)]">{{ JSON.stringify(latest, null, 2) }}</pre>
        <input v-model="tag" class="noro-input" placeholder="v1.2.3">
        <div class="flex justify-end gap-3 pt-2">
          <AtomButton variant="secondary" @click="showBuild = false">Cancel</AtomButton>
          <AtomButton
            variant="primary"
            icon="i-lucide-hammer"
            :disabled="busy === 'build' || !tag"
            @click="buildLauncher"
          >
            Build tag
          </AtomButton>
        </div>
      </div>
    </AtomModal>

    <LauncherBuildLog v-model="showLog" :job-id="jobId" />
  </NoroShell>
</template>
