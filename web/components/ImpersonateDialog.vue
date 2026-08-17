<script setup lang="ts">
const props = defineProps<{ userId: string; username: string }>()
const open = defineModel<boolean>({ required: true })

const auth = useAuth()
const { t } = useT()
const notify = useNotify()

const reason = ref('')
const code = ref('')
const busy = ref(false)
const needStepUp = ref(false)
const status = ref<string | null>(null)

async function stepUp() {
  busy.value = true
  try {
    await auth.request('/api/admin/step-up/recovery', {
      method: 'POST',
      body: { code: code.value.trim() },
    })
    needStepUp.value = false
    notify.ok('Confirmed for the next 15 minutes')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function start() {
  busy.value = true
  status.value = null
  try {
    const res = await auth.request<{ grant_id: string; launcher_online: boolean }>(
      `/api/admin/users/${props.userId}/impersonate`,
      { method: 'POST', body: { reason: reason.value.trim() } }
    )
    status.value = res.launcher_online
      ? 'Confirm the prompt in your launcher.'
      : 'Your launcher is offline — open it and confirm there.'
    poll(res.grant_id)
  } catch (e: any) {
    if (String(e?.data?.error || '').includes('step_up_required')) {
      needStepUp.value = true
    } else {
      notify.fail(e)
    }
  } finally {
    busy.value = false
  }
}

function poll(grantId: string) {
  const timer = setInterval(async () => {
    try {
      const res = await auth.request<{ status: string }>(`/api/admin/impersonate/${grantId}`)
      status.value = `Status: ${res.status}`
      if (res.status !== 'pending') clearInterval(timer)
    } catch {
      clearInterval(timer)
    }
  }, 1000)
  setTimeout(() => clearInterval(timer), 90_000)
}
</script>

<template>
  <NoroModal v-model="open" :title="t('admin-users-impersonate-title')">
    <div class="grid gap-4">
      <UAlert
        color="warning"
        variant="subtle"
        icon="i-lucide-eye-off"
        :description="t('admin-users-impersonate-warn')"
      />

      <template v-if="needStepUp">
        <label class="block">
          <span class="noro-label mb-1.5 block">{{ t('admin-users-impersonate-code') }}</span>
          <input v-model="code" class="noro-input w-full font-mono" placeholder="XXXX-XXXX-XXXX">
        </label>
        <AtomButton icon="i-lucide-shield-check" :loading="busy" @click="stepUp">{{ t('admin-users-impersonate-confirm') }}</AtomButton>
      </template>

      <template v-else>
        <label class="block">
          <span class="noro-label mb-1.5 block">{{ t('admin-users-reqlogs-why') }}</span>
          <input v-model="reason" class="noro-input w-full" placeholder="Ticket #123: items disappeared">
        </label>
        <AtomButton icon="i-lucide-user-check" :loading="busy" :disabled="reason.trim().length < 3" @click="start">
          {{ t('admin-users-impersonate-request', { name: username }) }}
        </AtomButton>
      </template>

      <p v-if="status" class="text-xs text-[var(--noro-muted)]">{{ status }}</p>
    </div>
  </NoroModal>
</template>
