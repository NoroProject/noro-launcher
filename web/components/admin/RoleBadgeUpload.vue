<script setup lang="ts">
/**
 * Своя картинка плашки роли.
 *
 * Нарисованная нами подходит не всем: у роли может быть свой знак, которого
 * шрифтом 5×7 не набрать. Загруженная заменяет плашку целиком — ни градиент,
 * ни текст к ней уже не подмешиваются.
 *
 * Требования к размеру показаны заранее, а не в ответе на неудачу: подбирать
 * картинку методом тыка — так себе занятие.
 */
const props = defineProps<{ roleId: string, hasImage: boolean }>()
const emit = defineEmits<{ changed: [] }>()

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const busy = ref(false)
const file = ref<HTMLInputElement>()

async function upload(event: Event) {
  const picked = (event.target as HTMLInputElement).files?.[0]
  if (!picked) return
  busy.value = true
  try {
    const body = new FormData()
    body.append('file', picked)
    await auth.request(`/api/admin/roles/${props.roleId}/badge`, { method: 'PUT', body })
    notify.ok(t('admin-role-badge-uploaded'), t('admin-role-saved-hint'))
    emit('changed')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
    if (file.value) file.value.value = ''
  }
}

async function clear() {
  busy.value = true
  try {
    await auth.request(`/api/admin/roles/${props.roleId}/badge`, { method: 'DELETE' })
    notify.ok(t('admin-role-badge-cleared'))
    emit('changed')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="grid gap-2">
    <span class="noro-label">{{ t('admin-role-badge-own') }}</span>

    <div class="flex flex-wrap items-center gap-2">
      <AtomButton
        variant="secondary"
        icon="i-lucide-upload"
        :loading="busy"
        @click="file?.click()"
      >{{ t('admin-role-badge-pick') }}</AtomButton>

      <AtomButton
        v-if="hasImage"
        variant="ghost"
        icon="i-lucide-x"
        :loading="busy"
        @click="clear"
      >{{ t('admin-role-badge-clear') }}</AtomButton>

      <input ref="file" type="file" accept="image/png" class="hidden" @change="upload">
    </div>

    <span class="text-xs leading-5 text-[var(--noro-muted)]">
      {{ t('admin-role-badge-rules') }}
    </span>
  </div>
</template>
