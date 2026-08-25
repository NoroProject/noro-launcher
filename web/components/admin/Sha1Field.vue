<script setup lang="ts">
/**
 * SHA1 запрещённого файла: руками или из образца.
 *
 * Хэш чит-мода админ обычно не знает — у него на руках сам jar, изъятый у
 * игрока. Считаем прямо в браузере: файл никуда не уходит, наружу отправляется
 * только сорок символов. Заодно это единственный способ не ошибиться, копируя
 * хэш из чужого сообщения.
 */
const sha1 = defineModel<string>({ required: true })
const { t } = useT()

const mode = ref<'manual' | 'file'>('manual')
const sample = ref<{ name: string, size: number } | null>(null)
const busy = ref(false)
const failed = ref('')

async function digest(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  busy.value = true
  failed.value = ''
  try {
    // Web Crypto живёт только в защищённом контексте: по http на чужом хосте
    // его нет, и честнее сказать об этом, чем оставить поле пустым.
    if (!globalThis.crypto?.subtle) {
      failed.value = t('admin-blocklist-sha1-no-crypto')
      return
    }
    const buffer = await file.arrayBuffer()
    const hash = await crypto.subtle.digest('SHA-1', buffer)
    sha1.value = [...new Uint8Array(hash)].map(b => b.toString(16).padStart(2, '0')).join('')
    sample.value = { name: file.name, size: file.size }
  } catch (e) {
    failed.value = String((e as Error).message || e)
  } finally {
    busy.value = false
  }
}

function reset() {
  sample.value = null
  sha1.value = ''
  failed.value = ''
}
</script>

<template>
  <div class="grid min-w-0 content-start">
    <div class="mb-1.5 flex items-baseline gap-3">
      <span class="noro-label !mb-0">{{ t('admin-blocklist-sha1') }}</span>
      <button
        v-for="option in (['manual', 'file'] as const)"
        :key="option"
        type="button"
        class="noro-label !mb-0 transition"
        :class="mode === option ? '!text-[var(--noro-cream)]' : 'hover:!text-[var(--noro-text)]'"
        @click="mode = option"
      >{{ t(`admin-blocklist-sha1-mode-${option}`) }}</button>
    </div>

    <input
      v-if="mode === 'manual'"
      v-model="sha1"
      class="noro-input w-full font-mono"
      :placeholder="t('admin-blocklist-sha1-placeholder')"
    >

    <template v-else>
      <label
        v-if="!sample"
        class="flex cursor-pointer items-center gap-2 border border-dashed border-[var(--noro-border)] px-3 py-2.5 text-sm text-[var(--noro-muted)] transition hover:border-[var(--noro-blue)]"
      >
        <UIcon :name="busy ? 'i-lucide-loader-circle' : 'i-lucide-upload'" class="size-4" :class="busy && 'animate-spin'" />
        <span class="truncate">{{ t('admin-blocklist-sha1-pick') }}</span>
        <input type="file" class="hidden" @change="digest">
      </label>

      <div v-else class="grid gap-1 border border-[var(--noro-border)] px-3 py-2">
        <div class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate text-sm text-white">{{ sample.name }}</span>
          <span class="shrink-0 text-xs text-[var(--noro-muted)]">{{ compactBytes(sample.size) }}</span>
          <button type="button" class="shrink-0 text-[var(--noro-muted)] hover:text-[var(--noro-cream)]" @click="reset">
            <UIcon name="i-lucide-x" class="size-4" />
          </button>
        </div>
        <code class="truncate font-mono text-xs text-[var(--noro-blue)]">{{ sha1 }}</code>
      </div>

      <p class="mt-1.5 text-[10px] leading-4 text-[var(--noro-muted)]">{{ t('admin-blocklist-sha1-local') }}</p>
    </template>

    <p v-if="failed" class="mt-1.5 text-xs text-[var(--noro-magenta)]">{{ failed }}</p>
  </div>
</template>
