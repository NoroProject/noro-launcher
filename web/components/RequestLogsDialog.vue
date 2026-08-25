<script setup lang="ts">
import type { ServerRow } from '~/types/api'

const props = defineProps<{ userId: string; username: string }>()
const open = defineModel<boolean>({ required: true })

const auth = useAuth()
const { t } = useT()
const notify = useNotify()

const reason = ref('')
const forced = ref(false)
const selectedServerId = ref('')
const servers = ref<ServerRow[]>([])
const busy = ref(false)
const status = ref<string | null>(null)

const canForce = computed(() => auth.hasPermission('noro.admin.support.logs.force'))

async function loadServers() {
  try {
    servers.value = (await auth.requestList<ServerRow>('/api/admin/servers')) ?? []
  } catch (e) {
    // Ignore error
  }
}

async function send() {
  busy.value = true
  status.value = null
  try {
    const res = await auth.request<{ launcher_online: boolean }>(
      `/api/admin/users/${props.userId}/request-logs`,
      {
        method: 'POST',
        body: {
          reason: reason.value.trim(),
          forced: forced.value,
          server_id: selectedServerId.value || undefined
        }
      }
    )
    status.value = res.launcher_online
      ? forced.value
        ? 'Collecting — the bundle will appear shortly.'
        : 'Waiting for the player to accept.'
      : 'The launcher is offline — queued for 24 hours.'
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

onMounted(() => {
  loadServers()
})

watch(open, (val) => {
  if (val && !servers.value.length) {
    loadServers()
  }
})
</script>

<template>
  <NoroModal v-model="open" :title="t('admin-users-reqlogs-title')">
    <div class="grid gap-4">
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-users-reqlogs-target') }}</span>
        <NoroSelect v-model="selectedServerId">
          <option value="">{{ t('admin-users-reqlogs-auto') }}</option>
          <option v-for="s in servers" :key="s.id" :value="s.id">
            {{ s.name }}
          </option>
        </NoroSelect>
        <span class="mt-1 block text-xs text-[var(--noro-muted)]">
          Specify a server to collect logs from its specific instance directory.
        </span>
      </label>

      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-users-reqlogs-why') }}</span>
        <input v-model="reason" class="noro-input w-full" placeholder="Ticket #123: the game crashes on launch">
        <span class="mt-1 block text-xs text-[var(--noro-muted)]">
          {{ t('admin-users-reqlogs-why-hint') }}
        </span>
      </label>

      <label v-if="canForce" class="flex items-start gap-2 text-xs text-[var(--noro-muted)]">
        <input v-model="forced" type="checkbox" class="mt-0.5 size-4">
        <span>
          <span class="font-bold text-[var(--noro-magenta)]">{{ t('admin-users-reqlogs-force') }}</span>
          {{ t('admin-users-reqlogs-force-hint') }}
        </span>
      </label>

      <AtomButton icon="i-lucide-file-text" :loading="busy" :disabled="reason.trim().length < 3" @click="send">
        {{ forced ? t('admin-users-reqlogs-btn-now', { name: username }) : t('admin-users-reqlogs-btn-ask', { name: username }) }}
      </AtomButton>

      <p v-if="status" class="text-xs text-[var(--noro-muted)]">{{ status }}</p>
    </div>
  </NoroModal>
</template>
