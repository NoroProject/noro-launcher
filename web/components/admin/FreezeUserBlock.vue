<script setup lang="ts">
const props = defineProps<{
  userId: string
  frozen?: boolean
}>()

const emit = defineEmits<{
  (e: 'updated'): void
}>()

const auth = useAuth()
const { t } = useT()
const notify = useNotify()

const reason = ref('')
const pending = ref(false)

async function freezePlayer() {
  pending.value = true
  try {
    await auth.request('/api/admin/freezes', {
      method: 'POST',
      body: { target: props.userId, reason: reason.value || 'Заморозка модератором' }
    })
    notify.ok()
    reason.value = ''
    emit('updated')
  } catch (e) {
    notify.fail(e)
  } finally {
    pending.value = false
  }
}

async function unfreezePlayer() {
  pending.value = true
  try {
    await auth.request(`/api/admin/freezes/${props.userId}`, { method: 'DELETE' })
    notify.ok()
    emit('updated')
  } catch (e) {
    notify.fail(e)
  } finally {
    pending.value = false
  }
}
</script>

<template>
  <div class="noro-panel p-5 space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-black text-[var(--noro-text)]">
        {{ t('admin-freezes-section-title') }}
      </h3>
      <UBadge :color="frozen ? 'error' : 'neutral'" variant="subtle">
        {{ frozen ? t('admin-freezes-frozen') : t('admin-freezes-active') }}
      </UBadge>
    </div>

    <div v-if="!frozen" class="space-y-3">
      <input
        v-model="reason"
        class="noro-input w-full text-xs"
        :placeholder="t('admin-freezes-reason-placeholder')"
      >
      <AtomButton variant="primary" icon="i-lucide-snowflake" :disabled="pending" @click="freezePlayer">
        {{ t('admin-freezes-action-freeze') }}
      </AtomButton>
    </div>

    <div v-else class="space-y-3">
      <AtomButton variant="dark" icon="i-lucide-sun" :disabled="pending" @click="unfreezePlayer">
        {{ t('admin-freezes-action-unfreeze') }}
      </AtomButton>
    </div>
  </div>
</template>
