<script setup lang="ts">
import { SETTING_LABELS, type SettingItem } from '~/types/settings'

const props = defineProps<{ item: SettingItem }>()
const model = defineModel<string>({ required: true })

const meta = computed(() => SETTING_LABELS[props.item.key])
</script>

<template>
  <label class="block">
    <span class="noro-label mb-1.5 flex items-center gap-2">
      {{ meta?.label || item.key }}
      <span
        v-if="item.from_env"
        class="rounded bg-[color-mix(in_srgb,var(--noro-cream)_16%,transparent)] px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider text-[var(--noro-cream)]"
        :title="`Set by ${item.env}. The environment wins over the database, so editing here changes nothing until you unset it.`"
      >from env</span>
    </span>

    <input
      v-model="model"
      class="noro-input w-full"
      :disabled="item.from_env"
      :placeholder="item.env"
    >

    <span v-if="meta?.hint" class="mt-1 block text-xs text-[var(--noro-muted)]">
      {{ meta.hint }}
    </span>
  </label>
</template>
