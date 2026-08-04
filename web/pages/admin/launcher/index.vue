<script setup lang="ts">
import type { LauncherVersionRow } from '~/types/api'

const auth = useAuth()
await auth.loadMe()

const { data: versions, refresh, pending, error } = await useAsyncData('admin-launcher-versions', () =>
  auth.request<LauncherVersionRow[]>('/api/admin/launcher/versions'), { default: () => [] }
)

const tag = ref('')
const latest = ref<Record<string, unknown> | null>(null)
const jobId = ref('')
const jobLog = ref<Record<string, unknown> | null>(null)
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
  } finally {
    busy.value = null
  }
}

async function loadLog() {
  if (!jobId.value) return
  busy.value = 'log'
  try {
    jobLog.value = await auth.request<Record<string, unknown>>(`/api/admin/launcher/build/${jobId.value}/log`)
  } finally {
    busy.value = null
  }
}

async function deploy(versionId: string) {
  busy.value = `deploy-${versionId}`
  try {
    await auth.request(`/api/admin/launcher/deploy/${versionId}`, { method: 'POST' })
    await refresh()
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
      <button type="button" class="noro-btn noro-btn-primary" @click="showBuild = true">
        <UIcon name="i-lucide-hammer" class="size-5" />Build tag
      </button>
      <button type="button" class="noro-btn noro-btn-secondary" @click="showLog = true">
        <UIcon name="i-lucide-file-text" class="size-5" />Build log
      </button>
    </template>

    <UAlert v-if="error" class="mb-5" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="humanError(error)" />

    <section class="noro-panel overflow-hidden">
      <table v-if="versions?.length" class="noro-table">
        <thead><tr><th>Version</th><th>Platform</th><th>SHA256</th><th>Current</th><th /></tr></thead>
        <tbody>
          <tr v-for="version in versions" :key="version.id">
            <td class="font-semibold text-[var(--noro-text)]">{{ version.version }}</td>
            <td>{{ version.platform }}</td>
            <td><code class="text-xs text-[var(--noro-muted)]">{{ version.sha256.slice(0, 16) }}...</code></td>
            <td><UBadge :color="version.is_current ? 'success' : 'neutral'" variant="subtle">{{ version.is_current ? 'current' : 'stored' }}</UBadge></td>
            <td class="text-right">
              <UButton :loading="busy === `deploy-${version.id}`" icon="i-lucide-send" color="primary" size="sm" variant="subtle" @click="deploy(version.id)">Deploy</UButton>
            </td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-else icon="i-lucide-rocket" title="No versions yet" text="Build a launcher tag from the toolbar." />
    </section>

    <AtomModal v-model="showBuild" title="GITHUB BUILD" subtitle="Build a launcher release tag">
      <div class="grid gap-3">
        <UButton :loading="busy === 'github'" icon="i-lucide-github" color="neutral" variant="subtle" @click="githubLatest">Check latest release</UButton>
        <pre v-if="latest" class="rounded-lg bg-[var(--noro-input)] p-3 text-xs text-[var(--noro-text)]">{{ JSON.stringify(latest, null, 2) }}</pre>
        <input v-model="tag" class="noro-input" placeholder="v1.2.3">
        <div class="flex justify-end gap-3 pt-2">
          <button type="button" class="noro-btn noro-btn-secondary" @click="showBuild = false">Cancel</button>
          <button
            type="button"
            class="noro-btn noro-btn-primary"
            :disabled="busy === 'build' || !tag"
            @click="buildLauncher"
          >
            <UIcon name="i-lucide-hammer" class="size-5" />Build tag
          </button>
        </div>
      </div>
    </AtomModal>

    <AtomModal v-model="showLog" title="BUILD LOG" subtitle="Inspect launcher builder output" wide>
      <div class="flex gap-2">
        <input v-model="jobId" class="noro-input" placeholder="job_id">
        <UButton :loading="busy === 'log'" icon="i-lucide-file-text" color="neutral" variant="subtle" @click="loadLog" />
      </div>
      <pre v-if="jobLog" class="mt-3 max-h-80 overflow-auto rounded-lg bg-[var(--noro-input)] p-3 text-xs text-[var(--noro-text)]">{{ JSON.stringify(jobLog, null, 2) }}</pre>
    </AtomModal>
  </NoroShell>
</template>
