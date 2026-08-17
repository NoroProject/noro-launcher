<script setup lang="ts">
import { INTEGRITY_LABELS, type IntegrityFlag } from '~/types/integrity'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()
await auth.loadMe()

const onlyOpen = ref(true)
const rows = ref<IntegrityFlag[]>([])
const pending = ref(false)

async function load() {
  pending.value = true
  try {
    const p = new URLSearchParams({ limit: '100' })
    if (onlyOpen.value) p.set('open', 'true')
    rows.value = await auth.request<IntegrityFlag[]>(`/api/admin/integrity?${p}`)
  } catch (e) {
    notify.fail(e, 'Failed to load integrity flags')
  } finally {
    pending.value = false
  }
}

async function review(id: number) {
  try {
    await auth.request(`/api/admin/integrity/${id}/review`, { method: 'POST' })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

/** Подложенный файл важнее пропавшего: второе бывает от битого диска. */
const severe = (kind: IntegrityFlag['kind']) =>
  kind === 'extra_file' || kind === 'forbidden_optional_mod'

watch(onlyOpen, () => load())
onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-integrity-title')" :subtitle="t('admin-integrity-subtitle')">
    <template #actions>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <NoroNote class="mb-4">
      {{ t('admin-integrity-note') }}
    </NoroNote>

    <label class="mb-4 flex items-center gap-2 text-xs text-[var(--noro-muted)]">
      <input v-model="onlyOpen" type="checkbox" class="size-4">
      {{ t('admin-integrity-unreviewed-only') }}
    </label>

    <section v-if="rows.length" class="noro-panel overflow-x-auto">
      <table class="noro-table">
        <thead>
          <tr>
            <th>{{ t('admin-integrity-col-when') }}</th>
            <th>{{ t('admin-integrity-col-finding') }}</th>
            <th>{{ t('admin-integrity-col-subject') }}</th>
            <th>{{ t('admin-integrity-col-build') }}</th>
            <th>{{ t('admin-integrity-col-player') }}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in rows" :key="row.id">
            <td class="whitespace-nowrap text-xs">{{ new Date(row.at).toLocaleString() }}</td>
            <td>
              <span
                class="rounded px-2 py-1 text-[10px] font-black uppercase tracking-wider"
                :class="severe(row.kind)
                  ? 'bg-[color-mix(in_srgb,var(--noro-magenta)_16%,transparent)] text-[var(--noro-magenta)]'
                  : 'bg-[color-mix(in_srgb,var(--noro-muted)_16%,transparent)] text-[var(--noro-muted)]'"
              >{{ INTEGRITY_LABELS[row.kind] }}</span>
              <span v-if="row.repaired" class="ml-2 text-[10px] text-[var(--noro-muted)]">{{ t('admin-integrity-repaired') }}</span>
            </td>
            <td class="font-mono text-xs">
              {{ row.subject }}
              <div v-if="row.detail" class="text-[10px] text-[var(--noro-muted)]">{{ row.detail }}</div>
            </td>
            <td class="whitespace-nowrap text-xs">
              {{ row.build_version || '—' }}
              <div class="text-[10px] text-[var(--noro-muted)]">{{ t('admin-integrity-launcher') }} {{ row.launcher_version || '—' }}</div>
            </td>
            <td>
              <NuxtLink :to="`/admin/users/${row.user_id}`" class="text-xs underline hover:text-[var(--noro-cream)]">
                {{ t('admin-integrity-open-card') }}
              </NuxtLink>
            </td>
            <td class="text-right">
              <AtomButton
                v-if="!row.reviewed_at && can('noro.admin.integrity.review')"
                variant="dark"
                icon="i-lucide-check"
                class="!min-h-8 !px-2"
                @click="review(row.id)"
              />
              <span v-else class="text-[10px] text-[var(--noro-muted)]">{{ t('admin-integrity-reviewed') }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </section>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-shield-check"
      :title="t('admin-integrity-empty-title')"
      :text="t('admin-integrity-empty-text')"
    />
  </NoroShell>
</template>
