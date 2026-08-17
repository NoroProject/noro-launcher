<script setup lang="ts">
export interface SupportBundle {
  id: string
  at: string
  user_id: string
  server_id: string | null
  note: string
  voluntary: boolean
  file_sha1: string
  size: number
  expires_at: string
}

export interface LogRequest {
  id: string
  actor_id: string | null
  actor_label: string
  target_id: string
  reason: string
  server_id: string | null
  forced: boolean
  status: 'pending' | 'accepted' | 'declined' | 'delivered' | 'expired'
  created_at: string
  expires_at: string
  answered_at: string | null
  bundle_id: string | null
}

const props = defineProps<{ userId: string }>()
const emit = defineEmits<{ requestLogs: [] }>()

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
      auth.request<SupportBundle[]>(`/api/admin/support/bundles?user_id=${props.userId}`),
      auth.request<LogRequest[]>(`/api/admin/support/requests?user_id=${props.userId}`)
    ])
    bundles.value = bRes
    logRequests.value = rRes
  } catch (e) {
    notify.fail(e, 'Failed to load support bundles')
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

const actionBusy = ref(false)

async function runAction(action: 'restart_launcher' | 'kill_game') {
  actionBusy.value = true
  try {
    await auth.request(`/api/admin/users/${props.userId}/action`, {
      method: 'POST',
      body: { action }
    })
    notify.ok(action === 'restart_launcher' ? 'Restart requested' : 'Kill game requested')
  } catch (e) {
    notify.fail(e, 'Failed to send action')
  } finally {
    actionBusy.value = false
  }
}

onMounted(() => load())
</script>

<template>
  <section class="noro-panel space-y-4 p-5">
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0">
        <h3 class="text-lg font-bold text-[var(--noro-text)]">{{ t('admin-support-panel-title') }}</h3>
        <p class="text-xs text-[var(--noro-muted)]">{{ t('admin-support-panel-subtitle') }}</p>
      </div>
      <AtomButton variant="dark" size="sm" icon="i-lucide-refresh-cw" :loading="pending" class="shrink-0" @click="load" />
    </div>

    <!-- Действия отдельным рядом: рядом с заголовком они в узкой колонке
         вставали лесенкой разной ширины и наезжали на подзаголовок. -->
    <div class="flex flex-wrap gap-2">
      <AtomButton v-if="can('noro.admin.support.request')" variant="dark" icon="i-lucide-file-text" size="sm" class="min-w-40 flex-1" @click="emit('requestLogs')">
        {{ t('admin-users-req-logs') }}
      </AtomButton>
      <AtomButton v-if="can('noro.admin.users.launcher')" variant="dark" icon="i-lucide-x-circle" size="sm" class="min-w-40 flex-1" :disabled="actionBusy" @click="runAction('kill_game')">
        {{ t('admin-support-close-game') }}
      </AtomButton>
      <AtomButton v-if="can('noro.admin.users.launcher')" variant="dark" icon="i-lucide-rotate-ccw" size="sm" class="min-w-40 flex-1" :disabled="actionBusy" @click="runAction('restart_launcher')">
        {{ t('admin-support-restart-launcher') }}
      </AtomButton>
    </div>

    <!-- Delivered Support Bundles (ZIP archives) -->
    <div class="space-y-2">
      <div class="noro-label flex items-center justify-between">
        <span>{{ t('admin-support-delivered-bundles', { count: bundles.length }) }}</span>
      </div>

      <div v-if="bundles.length" class="space-y-2">
        <div
          v-for="b in bundles"
          :key="b.id"
          class="flex flex-col justify-between gap-3 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] p-3 text-xs sm:flex-row sm:items-center"
        >
          <div class="min-w-0 space-y-1">
            <div class="flex flex-wrap items-center gap-2">
              <UIcon name="i-lucide-archive" class="size-4 shrink-0 text-[var(--noro-cream)]" />
              <span class="font-mono text-xs font-bold text-[var(--noro-text)]">{{ b.id.slice(0, 8) }}</span>
              <UBadge :color="b.voluntary ? 'neutral' : 'warning'" variant="subtle" class="text-[10px]">
                {{ b.voluntary ? t('admin-support-voluntary') : t('admin-support-forced') }}
              </UBadge>
              <span class="text-[var(--noro-muted)]">· {{ formatSize(b.size) }}</span>
            </div>
            <p v-if="b.note" class="truncate text-[var(--noro-text)]">{{ b.note }}</p>
            <div class="text-[10px] text-[var(--noro-muted)]">
              Received {{ new Date(b.at).toLocaleString() }} · Expires {{ new Date(b.expires_at).toLocaleDateString() }}
            </div>
          </div>

          <AtomButton
            v-if="can('noro.admin.support.download')"
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
      <EmptyState v-else icon="i-lucide-folder-archive" :title="t('admin-support-nobundles-title')" :text="t('admin-support-nobundles-player')" />
    </div>

    <!-- Log Requests History -->
    <div class="space-y-2 pt-2">
      <div class="noro-label">{{ t('admin-support-req-history', { count: logRequests.length }) }}</div>

      <div v-if="logRequests.length" class="max-h-80 overflow-y-auto space-y-2 pr-1">
        <div
          v-for="req in logRequests"
          :key="req.id"
          class="flex flex-col justify-between gap-2 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-3 text-xs sm:flex-row sm:items-center"
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
            <div class="text-[10px] text-[var(--noro-muted)]">
              By {{ req.actor_label }} · {{ new Date(req.created_at).toLocaleString() }}
            </div>
          </div>

          <div class="flex items-center gap-2">
            <AtomButton
              v-if="req.status === 'pending'"
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
              v-if="req.bundle_id"
              variant="secondary"
              size="sm"
              icon="i-lucide-download"
              :loading="downloadingId === req.bundle_id"
              class="shrink-0"
              @click="downloadBundle(req.bundle_id)"
            >
              {{ t('admin-support-download-zip') }}
            </AtomButton>
          </div>
        </div>
      </div>
      <EmptyState v-else icon="i-lucide-history" :title="t('admin-support-norequests-title')" :text="t('admin-support-norequests-player')" />
    </div>
  </section>
</template>
