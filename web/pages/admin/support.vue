<script setup lang="ts">
import type { SupportBundle, LogRequest } from '~/components/SupportBundlesPanel.vue'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()

const bundles = ref<SupportBundle[]>([])
const logRequests = ref<LogRequest[]>([])
const pending = ref(false)
const downloadingId = ref<string | null>(null)

async function load() {
  pending.value = true
  try {
    const [bRes, rRes] = await Promise.all([
      auth.request<SupportBundle[]>('/api/admin/support/bundles'),
      auth.request<LogRequest[]>('/api/admin/support/requests')
    ])
    bundles.value = bRes
    logRequests.value = rRes
  } catch (e) {
    notify.fail(e, 'Failed to load support logs')
  } finally {
    pending.value = false
  }
}

async function downloadBundle(bundleId: string) {
  downloadingId.value = bundleId
  try {
    const blob = await auth.request<Blob>(`/api/admin/support/bundles/${bundleId}`, {
      responseType: 'blob' as unknown as undefined
    })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `noro-bundle-${bundleId.slice(0, 8)}.zip`
    a.click()
    URL.revokeObjectURL(url)
    notify.ok('Archive downloaded')
  } catch (e) {
    notify.fail(e, 'Failed to download bundle')
  } finally {
    downloadingId.value = null
  }
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

function statusColor(status: string) {
  switch (status) {
    case 'delivered': return 'success'
    case 'accepted': return 'primary'
    case 'pending': return 'warning'
    case 'declined': return 'error'
    case 'expired': return 'neutral'
    default: return 'neutral'
  }
}

const cancellingId = ref<string | null>(null)

async function cancelRequest(reqId: string) {
  cancellingId.value = reqId
  try {
    await auth.request(`/api/admin/support/requests/${reqId}`, { method: 'DELETE' })
    notify.ok('Request cancelled')
    await load()
  } catch (e) {
    notify.fail(e, 'Failed to cancel request')
  } finally {
    cancellingId.value = null
  }
}

onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-support-title')" :subtitle="t('admin-support-subtitle')">
    <template #actions>
      <AtomButton variant="dark" icon="i-lucide-refresh-cw" :loading="pending" @click="load">{{ t('cabinet-apps-refresh') }}</AtomButton>
    </template>

    <div class="grid gap-6 xl:grid-cols-2 items-start">
      <!-- Delivered Support Bundles -->
      <section class="noro-panel flex flex-col space-y-4 p-5 max-h-[calc(100vh-200px)] min-h-[400px]">
        <div class="flex items-center justify-between shrink-0">
          <h2 class="text-lg font-bold text-[var(--noro-text)]">{{ t('admin-support-bundles-title', { count: bundles.length }) }}</h2>
          <UBadge color="primary" variant="subtle">{{ t('admin-support-archives-count', { count: bundles.length }) }}</UBadge>
        </div>

        <div v-if="bundles.length" class="space-y-3 overflow-y-auto pr-1 flex-1">
          <div
            v-for="b in bundles"
            :key="b.id"
            class="flex flex-col justify-between gap-3 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] p-4 text-xs sm:flex-row sm:items-center"
          >
            <div class="min-w-0 space-y-1">
              <div class="flex flex-wrap items-center gap-2">
                <UIcon name="i-lucide-archive" class="size-4 shrink-0 text-[var(--noro-cream)]" />
                <NuxtLink :to="`/admin/users/${b.user_id}`" class="font-mono text-xs font-bold text-[var(--noro-cream)] hover:underline">
                  {{ t('admin-support-user', { id: b.user_id.slice(0, 8) }) }}
                </NuxtLink>
                <UBadge :color="b.voluntary ? 'neutral' : 'warning'" variant="subtle" class="text-[10px]">
                  {{ b.voluntary ? t('admin-support-voluntary') : t('admin-support-forced') }}
                </UBadge>
                <span class="text-[var(--noro-muted)]">· {{ formatSize(b.size) }}</span>
              </div>
              <p v-if="b.note" class="truncate text-[var(--noro-text)]">{{ b.note }}</p>
              <div class="text-[10px] text-[var(--noro-muted)]">
                {{ t('admin-support-date', { at: new Date(b.at).toLocaleString(), expires: new Date(b.expires_at).toLocaleDateString() }) }}
              </div>
            </div>

            <AtomButton
              variant="primary"
              size="sm"
              icon="i-lucide-download"
              :loading="downloadingId === b.id"
              class="shrink-0"
              @click="downloadBundle(b.id)"
            >
              {{ t('admin-support-download-zip') }}
            </AtomButton>
          </div>
        </div>
        <EmptyState v-else icon="i-lucide-folder-archive" :title="t('admin-support-nobundles-title')" :text="t('admin-support-nobundles-text')" />
      </section>

      <!-- Log Requests History -->
      <section class="noro-panel flex flex-col space-y-4 p-5 max-h-[calc(100vh-200px)] min-h-[400px]">
        <div class="flex items-center justify-between shrink-0">
          <h2 class="text-lg font-bold text-[var(--noro-text)]">{{ t('admin-support-requests-title', { count: logRequests.length }) }}</h2>
          <UBadge color="neutral" variant="subtle">{{ t('admin-support-requests-count', { count: logRequests.length }) }}</UBadge>
        </div>

        <div v-if="logRequests.length" class="space-y-3 overflow-y-auto pr-1 flex-1">
          <div
            v-for="req in logRequests"
            :key="req.id"
            class="flex flex-col justify-between gap-3 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-4 text-xs sm:flex-row sm:items-center"
          >
            <div class="min-w-0 space-y-1">
              <div class="flex flex-wrap items-center gap-2">
                <span class="font-bold text-[var(--noro-text)]">{{ req.reason }}</span>
                <UBadge :color="statusColor(req.status)" variant="subtle" class="text-[10px]">
                  {{ req.status }}
                </UBadge>
                <UBadge v-if="req.forced" color="error" variant="subtle" class="text-[10px]">
                  forced
                </UBadge>
              </div>
              <div class="text-[10px] text-[var(--noro-muted)] flex items-center gap-2">
                <span>By {{ req.actor_label }}</span>
                <span>·</span>
                <NuxtLink :to="`/admin/users/${req.target_id}`" class="text-[var(--noro-cream)] hover:underline">
                  Target {{ req.target_id.slice(0, 8) }}
                </NuxtLink>
                <span>· {{ new Date(req.created_at).toLocaleString() }}</span>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <AtomButton
                v-if="req.status === 'pending' && can('noro.admin.support.request')"
                variant="danger"
                size="sm"
                icon="i-lucide-x"
                :loading="cancellingId === req.id"
                class="shrink-0"
                @click="cancelRequest(req.id)"
              >
                {{ t('web-rules-cancel') }}
              </AtomButton>

              <AtomButton
                v-if="req.bundle_id && can('noro.admin.support.download')"
                variant="secondary"
                size="sm"
                icon="i-lucide-download"
                :loading="downloadingId === req.bundle_id"
                class="shrink-0"
                @click="downloadBundle(req.bundle_id)"
              >
                Download
              </AtomButton>
            </div>
          </div>
        </div>
        <EmptyState v-else icon="i-lucide-history" :title="t('admin-support-norequests-title')" :text="t('admin-support-norequests-text')" />
      </section>
    </div>
  </NoroShell>
</template>
