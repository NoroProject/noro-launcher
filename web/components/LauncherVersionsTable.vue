<script setup lang="ts">
import type { LauncherVersionRow } from '~/types/api'

const props = defineProps<{
  versions: LauncherVersionRow[]
  busy: string | null
}>()

defineEmits<{ deploy: [id: string], deployMany: [ids: string[], kind: string] }>()
const { t } = useT()

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
  // Выкаченное деплоить незачем — кнопки группы двигают только остальное.
  const pending = (items: LauncherVersionRow[], kind: string) =>
    items.filter(i => i.kind === kind && !i.is_current).map(i => i.id)
  return [...map].map(([version, items]) => ({
    version,
    items: [...items].sort(
      (a, b) => a.platform.localeCompare(b.platform) || a.kind.localeCompare(b.kind)
    ),
    // Порознь: core едет каждый релиз, а установщик обычно остаётся на старом —
    // общая кнопка «выкатить всё» сбрасывала его репутацию у SmartScreen.
    core: pending(items, 'core'),
    bootstrapper: pending(items, 'bootstrapper'),
  }))
})

// Раскрыта свежая версия, старые свёрнуты: разворачивать десять строк ради
// давно выкаченной сборки незачем. Выбор переживает refresh таблицы.
const expanded = ref(new Set<string>())
const touched = ref(false)
watchEffect(() => {
  const first = groups.value[0]?.version
  if (!touched.value && first) expanded.value = new Set([first])
})

function toggle(version: string) {
  touched.value = true
  const next = new Set(expanded.value)
  next.has(version) ? next.delete(version) : next.add(version)
  expanded.value = next
}
const auth = useAuth()
const can = (perm: string) => auth.hasPermission(perm)
</script>

<template>
  <table class="noro-table">
    <thead>
      <tr>
        <th>{{ t('admin-launchver-plat') }}</th>
        <th>{{ t('admin-launchver-kind') }}</th>
        <th>SHA256</th>
        <th>{{ t('admin-launchver-curr') }}</th>
        <th />
      </tr>
    </thead>
    <tbody v-for="group in groups" :key="group.version">
      <tr>
        <th colspan="5" class="bg-[var(--noro-input)] px-4 py-2">
          <div class="flex items-center gap-3">
            <button class="flex flex-1 items-center gap-2 text-left" @click="toggle(group.version)">
              <UIcon
                :name="expanded.has(group.version) ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'"
                class="size-4 text-[var(--noro-muted)]"
              />
              <span class="text-[var(--noro-text)]">{{ group.version }}</span>
              <span class="text-xs font-normal text-[var(--noro-muted)]">
                {{ t('admin-launchver-builds', { count: group.items.length }) }}
              </span>
            </button>
            <AtomButton
              v-if="group.core.length && can('noro.admin.launcher.deploy')"
              variant="secondary"
              size="sm"
              icon="i-lucide-send"
              :loading="busy === 'deploy-core'"
              @click="$emit('deployMany', group.core, 'core')"
            >
              {{ t('admin-launchver-deploy-core') }}
            </AtomButton>
            <AtomButton
              v-if="group.bootstrapper.length && can('noro.admin.launcher.deploy')"
              variant="dark"
              size="sm"
              icon="i-lucide-send"
              :loading="busy === 'deploy-bootstrapper'"
              @click="$emit('deployMany', group.bootstrapper, 'bootstrapper')"
            >
              {{ t('admin-launchver-deploy-boot') }}
            </AtomButton>
          </div>
        </th>
      </tr>

      <template v-if="expanded.has(group.version)">
        <tr v-for="version in group.items" :key="version.id">
          <td>{{ version.platform }}</td>
          <td class="text-xs text-[var(--noro-muted)]">{{ version.kind }}</td>
          <td><code class="text-xs text-[var(--noro-muted)]">{{ version.sha256.slice(0, 16) }}...</code></td>
          <td>
            <UBadge :color="version.is_current ? 'success' : 'neutral'" variant="subtle">
              {{ version.is_current ? t('admin-launchver-is-curr') : t('admin-launchver-stored') }}
            </UBadge>
          </td>
          <td class="text-right">
            <AtomButton
              v-if="!version.is_current && can('noro.admin.launcher.deploy')"
              variant="primary"
              size="sm"
              icon="i-lucide-send"
              :loading="busy === `deploy-${version.id}`"
              @click="$emit('deploy', version.id)"
            >
              {{ t('admin-launchver-deploy') }}
            </AtomButton>
          </td>
        </tr>
      </template>
    </tbody>
  </table>
</template>
