<script setup lang="ts">
import { SETTING_LABELS, type SettingItem } from '~/types/settings'

const props = defineProps<{ item: SettingItem }>()
const model = defineModel<string>({ required: true })
const { t } = useT()

const meta = computed(() => SETTING_LABELS[props.item.key])
const label = computed(() => {
  const key = `admin-set-label-${props.item.key}`
  const val = t(key)
  return val !== key ? val : (meta.value?.label || props.item.key)
})
const hint = computed(() => {
  const key = `admin-set-hint-${props.item.key}`
  const val = t(key)
  return val !== key ? val : meta.value?.hint
})
</script>

<template>
  <label class="block">
    <span class="noro-label mb-1.5 flex items-center gap-2">
      {{ label }}
      <span
        v-if="item.from_env"
        class="rounded bg-[color-mix(in_srgb,var(--noro-cream)_16%,transparent)] px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider text-[var(--noro-cream)]"
        :title="t('admin-set-from-env-title', { env: item.env })"
      >{{ t('admin-set-from-env') }}</span>
    </span>

    <input
      v-model="model"
      class="noro-input w-full"
      :disabled="item.from_env"
      :placeholder="item.env"
    >

    <span v-if="hint" class="mt-1 block text-xs text-[var(--noro-muted)]">
      {{ hint }}
    </span>
  </label>
</template>
