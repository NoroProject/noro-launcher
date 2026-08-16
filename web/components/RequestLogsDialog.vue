<script setup lang="ts">
const props = defineProps<{ userId: string; username: string }>()
const open = defineModel<boolean>({ required: true })

const auth = useAuth()
const notify = useNotify()

const reason = ref('')
const forced = ref(false)
const busy = ref(false)
const status = ref<string | null>(null)

/** Принудительный сбор виден только тому, у кого есть на него право. */
const canForce = computed(() => auth.hasPermission('noro.admin.support.logs.force'))

async function send() {
  busy.value = true
  status.value = null
  try {
    const res = await auth.request<{ launcher_online: boolean }>(
      `/api/admin/users/${props.userId}/request-logs`,
      { method: 'POST', body: { reason: reason.value.trim(), forced: forced.value } }
    )
    status.value = res.launcher_online
      ? forced.value
        ? 'Collecting — the bundle will appear shortly.'
        : 'Waiting for the player to accept.'
      : 'The launcher is offline — the request expires in 5 minutes.'
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <UModal v-model:open="open" title="Request logs">
    <template #body>
      <div class="grid gap-4">
        <label class="block">
          <span class="noro-label mb-1.5 block">Why</span>
          <input v-model="reason" class="noro-input w-full" placeholder="Ticket #123: the game crashes on launch">
          <span class="mt-1 block text-xs text-[var(--noro-muted)]">
            The player sees this text, and it stays in the audit log.
          </span>
        </label>

        <label v-if="canForce" class="flex items-start gap-2 text-xs text-[var(--noro-muted)]">
          <input v-model="forced" type="checkbox" class="mt-0.5 size-4">
          <span>
            <span class="font-bold text-[var(--noro-magenta)]">Collect without asking.</span>
            For investigations, where consent is meaningless — otherwise the only
            logs that never arrive are the ones you need. Recorded separately in
            the audit log.
          </span>
        </label>

        <AtomButton icon="i-lucide-file-text" :loading="busy" :disabled="reason.trim().length < 3" @click="send">
          {{ forced ? 'Collect now' : 'Ask' }} — {{ username }}
        </AtomButton>

        <p v-if="status" class="text-xs text-[var(--noro-muted)]">{{ status }}</p>
      </div>
    </template>
  </UModal>
</template>
