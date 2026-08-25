<script setup lang="ts">
import type { ServerRow } from '~/types/api'

const props = defineProps<{
  username: string
  userId: string
  targetUuid: string
}>()

const auth = useAuth()
const { t } = useT()
const notify = useNotify()

interface PlaySession {
  id: string
  server_id: string
  started_at: string
  ended_at: string | null
}

const activeSession = ref<PlaySession | null>(null)
const activeServerName = ref<string>('')
const isOnline = ref(false)
/// `false` — мастер не ответил, и про игрока мы ничего не знаем.
const statusKnown = ref(true)
const loading = ref(false)

const kickMessage = ref('')
const tellMessage = ref('')

const busyKick = ref(false)
const busyTell = ref(false)

async function checkOnlineStatus() {
  loading.value = true
  statusKnown.value = true
  try {
    // Без запасных пустых списков: неудавшийся запрос давал `sessions = []`,
    // из чего панель делала вывод «игрок офлайн» и гасила кнопки. Модератор
    // видел уверенное «Offline» там, где мастер просто не ответил.
    const [sessions, servers] = await Promise.all([
      auth.request<PlaySession[]>(`/api/admin/users/${props.userId}/play-sessions`),
      auth.requestList<ServerRow>('/api/admin/servers'),
    ])
    const open = sessions.find(s => s.ended_at === null)
    if (open) {
      activeSession.value = open
      isOnline.value = true
      const targetServer = servers.find(srv => srv.id === open.server_id)
      activeServerName.value = targetServer?.name || 'Server'
    } else {
      activeSession.value = null
      activeServerName.value = ''
      isOnline.value = false
    }
  } catch (e) {
    // Состояние неизвестно — это не то же самое, что «офлайн».
    statusKnown.value = false
    isOnline.value = false
    notify.fail(e)
  } finally {
    loading.value = false
  }
}

async function doKick() {
  busyKick.value = true
  try {
    await auth.request('/api/admin/game/kick', {
      method: 'POST',
      body: {
        server_id: activeSession.value?.server_id || undefined,
        target: props.targetUuid,
        message: kickMessage.value.trim() || t('admin-game-actions-kick-placeholder'),
      },
    })
    notify.ok()
    kickMessage.value = ''
    await checkOnlineStatus()
  } catch (e) {
    notify.fail(e)
  } finally {
    busyKick.value = false
  }
}

async function doTell() {
  if (!tellMessage.value.trim()) return
  busyTell.value = true
  try {
    await auth.request('/api/admin/game/tell', {
      method: 'POST',
      body: {
        server_id: activeSession.value?.server_id || undefined,
        target: props.targetUuid,
        message: tellMessage.value.trim(),
      },
    })
    notify.ok()
    tellMessage.value = ''
  } catch (e) {
    notify.fail(e)
  } finally {
    busyTell.value = false
  }
}

const can = (perm: string) => auth.hasPermission(perm)

onMounted(() => checkOnlineStatus())
</script>

<template>
  <div class="noro-panel p-5 space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between gap-4 border-b border-[var(--noro-border)] pb-3">
      <div>
        <h3 class="text-lg font-black text-[var(--noro-text)]">
          {{ t('admin-game-actions-title') }}
        </h3>
        <p class="text-xs text-[var(--noro-muted)]">
          {{ t('admin-game-actions-subtitle', { username }) }}
        </p>
      </div>
      <div class="flex items-center gap-3">
        <UBadge :color="!statusKnown ? 'warning' : isOnline ? 'success' : 'neutral'" variant="subtle">
          {{ !statusKnown ? t('admin-game-actions-status-unknown') : isOnline ? t('admin-game-actions-status-online', { server: activeServerName }) : t('admin-game-actions-status-offline') }}
        </UBadge>
        <AtomButton variant="dark" icon="i-lucide-refresh-cw" :loading="loading" @click="checkOnlineStatus" />
      </div>
    </div>

    <!-- Offline Banner -->
    <div v-if="!isOnline && statusKnown && !loading" class="rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] p-3 text-xs text-[var(--noro-muted)]">
      {{ t('admin-game-actions-offline-text', { username }) }}
    </div>

    <!-- Actions Grid -->
    <div
      class="grid gap-5 md:grid-cols-2"
      :class="!isOnline ? 'opacity-50 pointer-events-none' : ''"
    >
      <!-- Kick Card -->
      <div v-if="can('noro.admin.game.kick')" class="space-y-3 rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4">
        <div>
          <div class="font-bold text-[var(--noro-text)] text-sm">
            {{ t('admin-game-actions-kick-title', { username }) }}
          </div>
          <div class="text-[11px] text-[var(--noro-muted)]">
            {{ t('admin-game-actions-kick-desc') }}
          </div>
        </div>
        <div class="space-y-1">
          <span class="noro-label">{{ t('admin-game-actions-kick-reason') }}</span>
          <input
            v-model="kickMessage"
            class="noro-input w-full text-xs"
            :placeholder="t('admin-game-actions-kick-placeholder')"
            :disabled="!isOnline"
          >
        </div>
        <AtomButton
          variant="danger"
          :loading="busyKick"
          :disabled="!isOnline"
          @click="doKick"
        >
          {{ t('admin-game-actions-kick-btn') }}
        </AtomButton>
      </div>

      <!-- Tell / Direct Message Card -->
      <div v-if="can('noro.admin.game.tell')" class="space-y-3 rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4">
        <div>
          <div class="font-bold text-[var(--noro-text)] text-sm">
            {{ t('admin-game-actions-tell-title') }}
          </div>
          <div class="text-[11px] text-[var(--noro-muted)]">
            {{ t('admin-game-actions-tell-desc', { username }) }}
          </div>
        </div>
        <div class="space-y-1">
          <span class="noro-label">{{ t('admin-game-actions-tell-msg') }}</span>
          <input
            v-model="tellMessage"
            class="noro-input w-full text-xs"
            :placeholder="t('admin-game-actions-tell-placeholder')"
            :disabled="!isOnline"
          >
        </div>
        <AtomButton
          variant="primary"
          :loading="busyTell"
          :disabled="!isOnline || !tellMessage.trim()"
          @click="doTell"
        >
          {{ t('admin-game-actions-tell-btn') }}
        </AtomButton>
      </div>
    </div>
  </div>
</template>
