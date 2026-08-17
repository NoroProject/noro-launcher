<script setup lang="ts">
/**
 * Допустимые наказания за правило: вид и вилка срока.
 *
 * Вилка, а не одно число — правило описывает, что вообще можно, а модератор
 * выбирает внутри неё. Пустой список означает, что свод ничего не предписывает,
 * и наказать по этому правилу сможет только тот, у кого есть байпас.
 */
export interface SanctionDraft {
  kind: string
  label: string
  min: string
  max: string
}

const items = defineModel<SanctionDraft[]>({ required: true })
const { t } = useT()

const KINDS = computed(() => [
  ['warn', t('punish-warn')],
  ['mute', t('punish-mute')],
  ['ban', t('punish-ban')],
  ['server_ban', t('punish-server-ban')],
])

function add() {
  items.value = [...items.value, { kind: 'mute', label: '', min: '', max: '' }]
}

function remove(index: number) {
  items.value = items.value.filter((_, i) => i !== index)
}

/** Пустое поле — открытая граница, мусор — ошибка, а не «навсегда». */
function broken(value: string) {
  return !!value.trim() && Number.isNaN(parseDuration(value))
}
</script>

<template>
  <div class="grid gap-3">
    <div class="flex items-baseline justify-between gap-3">
      <span class="noro-label">{{ t('admin-sanc-title') }}</span>
      <AtomButton size="sm" variant="secondary" icon="i-lucide-plus" @click="add">{{ t('admin-sanc-add-option') }}</AtomButton>
    </div>

    <p v-if="!items.length" class="text-sm font-medium text-[var(--noro-muted)]">
      {{ t('admin-sanc-no-limits') }}
    </p>

    <div
      v-for="(item, index) in items"
      :key="index"
      class="grid gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] p-3 sm:grid-cols-[150px_1fr_1fr_auto]"
    >
      <label class="block">
        <span class="noro-label mb-2 block">{{ t('admin-sanc-kind') }}</span>
        <NoroSelect v-model="item.kind" class="w-full">
          <option v-for="[value, label] in KINDS" :key="value" :value="value">{{ label }}</option>
        </NoroSelect>
      </label>

      <label class="block" :class="item.kind === 'warn' ? 'opacity-40' : ''">
        <span class="noro-label mb-2 block">{{ t('admin-sanc-from') }}</span>
        <input
          v-model="item.min"
          class="noro-input w-full"
          :class="broken(item.min) ? '!border-[var(--noro-magenta)]' : ''"
          :disabled="item.kind === 'warn'"
          :placeholder="t('admin-sanc-min-placeholder')"
        >
      </label>

      <label class="block" :class="item.kind === 'warn' ? 'opacity-40' : ''">
        <span class="noro-label mb-2 block">{{ t('admin-sanc-to') }}</span>
        <input
          v-model="item.max"
          class="noro-input w-full"
          :class="broken(item.max) ? '!border-[var(--noro-magenta)]' : ''"
          :disabled="item.kind === 'warn'"
          :placeholder="t('admin-sanc-max-placeholder')"
        >
      </label>

      <div class="flex items-end">
        <AtomButton size="sm" variant="ghost" icon="i-lucide-trash-2" aria-label="Remove option"
                    @click="remove(index)" />
      </div>

      <label class="block sm:col-span-4">
        <span class="noro-label mb-2 block">{{ t('admin-sanc-note') }}</span>
        <input v-model="item.label" class="noro-input w-full" :placeholder="t('admin-sanc-label-placeholder')">
      </label>
    </div>
  </div>
</template>
