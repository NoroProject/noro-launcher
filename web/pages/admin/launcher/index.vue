<script setup lang="ts">
import type { LauncherVersionRow } from '~/types/api'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)

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

async function deployAll(ids: string[], kind: string) {
  busy.value = `deploy-${kind}`
  try {
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
  <NoroShell :title="t('admin-launch-title')" :subtitle="t('admin-launch-subtitle')">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
      <AtomButton v-if="can('noro.admin.launcher.publish')" variant="primary" icon="i-lucide-hammer" @click="showBuild = true">{{ t('admin-launch-build-tag') }}</AtomButton>
      <AtomButton variant="secondary" icon="i-lucide-file-text" @click="showLog = true">{{ t('admin-launch-build-log') }}</AtomButton>
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
      <EmptyState v-else icon="i-lucide-rocket" :title="t('admin-launch-empty-title')" :text="t('admin-launch-empty-text')" />
    </section>

    <AtomModal v-model="showBuild" :title="t('admin-launch-modal-title')" :subtitle="t('admin-launch-modal-subtitle')">
      <div class="grid gap-3">
        <AtomButton variant="secondary" :loading="busy === 'github'" icon="i-lucide-github" @click="githubLatest">{{ t('admin-launch-check-release') }}</AtomButton>
        <pre v-if="latest" class="rounded-lg bg-[var(--noro-input)] p-3 text-xs text-[var(--noro-text)]">{{ JSON.stringify(latest, null, 2) }}</pre>
        <input v-model="tag" class="noro-input" placeholder="v1.2.3">
        <div class="flex justify-end gap-3 pt-2">
          <AtomButton variant="secondary" @click="showBuild = false">{{ t('web-rules-cancel') }}</AtomButton>
          <AtomButton
            variant="primary"
            icon="i-lucide-hammer"
            :disabled="busy === 'build' || !tag"
            @click="buildLauncher"
          >
            {{ t('admin-launch-build-tag') }}
          </AtomButton>
        </div>
      </div>
    </AtomModal>

    <LauncherBuildLog v-model="showLog" :job-id="jobId" />
  </NoroShell>
</template>
