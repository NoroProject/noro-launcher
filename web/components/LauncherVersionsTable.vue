<script setup lang="ts">
import type { LauncherVersionRow } from '~/types/api'

const props = defineProps<{
  versions: LauncherVersionRow[]
  busy: string | null
}>()

defineEmits<{ deploy: [id: string] }>()

/**
 * Одна версия даёт десять строк: пять платформ, и у каждой две разновидности —
 * `bootstrapper` качает игрок, `core` качает сам bootstrapper. Без группировки и
 * колонки Kind это читается как дубликаты.
 */
const groups = computed(() => {
  const map = new Map<string, LauncherVersionRow[]>()
  for (const version of props.versions) {
    const list = map.get(version.version) || []
    list.push(version)
    map.set(version.version, list)
  }
  return [...map].map(([version, items]) => ({
    version,
    items: [...items].sort(
      (a, b) => a.platform.localeCompare(b.platform) || a.kind.localeCompare(b.kind)
    ),
  }))
})
</script>

<template>
  <table class="noro-table">
    <thead>
      <tr><th>Platform</th><th>Kind</th><th>SHA256</th><th>Current</th><th /></tr>
    </thead>
    <tbody v-for="group in groups" :key="group.version">
      <tr>
        <th colspan="5" class="bg-[var(--noro-input)] text-left text-[var(--noro-text)]">
          {{ group.version }}
        </th>
      </tr>
      <tr v-for="version in group.items" :key="version.id">
        <td>{{ version.platform }}</td>
        <td class="text-xs text-[var(--noro-muted)]">{{ version.kind }}</td>
        <td><code class="text-xs text-[var(--noro-muted)]">{{ version.sha256.slice(0, 16) }}...</code></td>
        <td>
          <UBadge :color="version.is_current ? 'success' : 'neutral'" variant="subtle">
            {{ version.is_current ? 'current' : 'stored' }}
          </UBadge>
        </td>
        <td class="text-right">
          <!-- У выкаченного деплоить нечего: кнопка остаётся только у остальных.
               Core и bootstrapper переезжают порознь — установщик можно держать
               на старой версии, пока он копит репутацию SmartScreen. -->
          <AtomButton
            v-if="!version.is_current"
            variant="primary"
            size="sm"
            icon="i-lucide-send"
            :loading="busy === `deploy-${version.id}`"
            @click="$emit('deploy', version.id)"
          >
            Deploy
          </AtomButton>
        </td>
      </tr>
    </tbody>
  </table>
</template>
