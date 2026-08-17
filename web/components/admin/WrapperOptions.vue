<script setup lang="ts">
const { t } = useT()

const RAW_OPTIONS = [
  { key: 'master-url', fallback: '—', required: true },
  { key: 'secret', fallback: '—', required: true },
  { key: 'server-jar', fallback: '—', required: true },
  { key: 'signing-public-key', fallbackKey: 'admin-wrapper-fb-fetch-pin' },
  { key: 'server-dir', fallback: '.' },
  { key: 'java', fallback: 'java' },
  { key: 'jvm-args', fallback: '-Xmx4G' },
  { key: 'server-args', fallback: 'nogui' },
  { key: 'platform', fallbackKey: 'admin-wrapper-fb-detected' },
  { key: 'mc-version', fallbackKey: 'admin-wrapper-fb-detected' },
]

/** Полный набор ключей noro-wrapper.properties — то же, что читает WrapperConfig. */
const OPTIONS = computed(() =>
  RAW_OPTIONS.map(opt => ({
    key: opt.key,
    fallback: opt.fallbackKey ? t(opt.fallbackKey) : opt.fallback,
    required: opt.required,
    note: t(`admin-wrapper-opt-${opt.key}`),
  }))
)
</script>

<template>
  <section class="noro-panel p-5">
    <div class="mb-4 flex items-center gap-3">
      <div class="grid size-10 place-items-center rounded bg-[var(--noro-amber)]/10 text-[var(--noro-amber)]">
        <UIcon name="i-lucide-sliders-horizontal" class="size-5" />
      </div>
      <div>
        <h3 class="text-lg font-black text-white">noro-wrapper.properties</h3>
        <p class="text-sm text-[var(--noro-muted)]">{{ t('admin-wrapper-subtitle') }}</p>
      </div>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full border-collapse text-sm">
        <thead>
          <tr class="border-b border-[var(--noro-border)] text-left text-xs uppercase text-[var(--noro-muted)]">
            <th class="py-2 pr-4 font-bold">{{ t('admin-wrapper-key') }}</th>
            <th class="py-2 pr-4 font-bold">{{ t('admin-wrapper-default') }}</th>
            <th class="py-2 font-bold">{{ t('admin-wrapper-meaning') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="option in OPTIONS"
            :key="option.key"
            class="border-b border-[var(--noro-border-soft)] align-top last:border-0"
          >
            <td class="py-3 pr-4 font-mono text-xs whitespace-nowrap text-[var(--noro-cream)]">
              {{ option.key }}
            </td>
            <td class="py-3 pr-4 text-xs whitespace-nowrap">
              <span v-if="option.required" class="font-bold text-[var(--noro-amber)]">{{ t('admin-wrapper-required') }}</span>
              <span v-else class="font-mono text-[var(--noro-muted)]">{{ option.fallback }}</span>
            </td>
            <td class="py-3 text-[var(--noro-muted)]">{{ option.note }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
