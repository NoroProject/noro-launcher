<script setup lang="ts">
interface BlockedFile {
  id: string
  pattern: string | null
  sha1: string | null
  reason: string
  action: 'delete' | 'flag' | 'block_launch'
  server_id: string | null
  created_at: string
}

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()
await auth.loadMe()

const rows = ref<BlockedFile[]>([])
const pattern = ref('')
const sha1 = ref('')
const reason = ref('')
const action = ref<BlockedFile['action']>('delete')
const pending = ref(false)

const load = async () => {
  rows.value = await auth.request<BlockedFile[]>('/api/admin/blocklist')
}

async function add() {
  pending.value = true
  try {
    await auth.request('/api/admin/blocklist', {
      method: 'POST',
      body: {
        pattern: pattern.value.trim() || null,
        sha1: sha1.value.trim() || null,
        reason: reason.value.trim(),
        action: action.value,
      },
    })
    pattern.value = ''
    sha1.value = ''
    reason.value = ''
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    pending.value = false
  }
}

async function remove(id: string) {
  try {
    await auth.request(`/api/admin/blocklist/${id}`, { method: 'DELETE' })
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

onMounted(() => load())
</script>

<template>
  <NoroShell :title="t('admin-blocklist-title')" :subtitle="t('admin-blocklist-subtitle')">
    <NoroNote class="mb-4">
      {{ t('admin-blocklist-note') }}
    </NoroNote>

    <section class="noro-panel mb-4 grid gap-4 p-4 md:grid-cols-[1fr_1fr_1fr_140px_auto]">
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-blocklist-mask') }}</span>
        <input v-model="pattern" class="noro-input w-full" :placeholder="t('admin-blocklist-mask-placeholder')">
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-blocklist-sha1') }}</span>
        <input v-model="sha1" class="noro-input w-full font-mono" :placeholder="t('admin-blocklist-sha1-placeholder')">
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-blocklist-reason') }}</span>
        <input v-model="reason" class="noro-input w-full" :placeholder="t('admin-blocklist-reason-placeholder')">
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">{{ t('admin-blocklist-action') }}</span>
        <NoroSelect v-model="action" class="w-full">
          <option value="delete">{{ t('admin-blocklist-act-delete') }}</option>
          <option value="flag">{{ t('admin-blocklist-act-flag') }}</option>
          <option value="block_launch">{{ t('admin-blocklist-act-block') }}</option>
        </NoroSelect>
      </label>
      <div class="flex items-end">
        <AtomButton v-if="can('noro.admin.blocklist.edit')" icon="i-lucide-plus" :loading="pending" @click="add">
          {{ t('admin-notes-add') }}
        </AtomButton>
      </div>
    </section>

    <section v-if="rows.length" class="noro-panel overflow-x-auto">
      <table class="noro-table">
        <thead>
          <tr>
            <th>{{ t('admin-blocklist-mask') }}</th>
            <th>{{ t('admin-blocklist-sha1') }}</th>
            <th>{{ t('admin-blocklist-reason') }}</th>
            <th>{{ t('admin-blocklist-action') }}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in rows" :key="r.id">
            <td class="font-mono text-xs">{{ r.pattern || '—' }}</td>
            <td class="font-mono text-xs">{{ r.sha1 ? `${r.sha1.slice(0, 12)}…` : '—' }}</td>
            <td class="text-xs">{{ r.reason }}</td>
            <td>
              <span
                class="rounded px-2 py-1 text-[10px] font-black uppercase tracking-wider"
                :class="r.action === 'block_launch'
                  ? 'bg-[color-mix(in_srgb,var(--noro-magenta)_16%,transparent)] text-[var(--noro-magenta)]'
                  : 'bg-[var(--noro-input)] text-[var(--noro-muted)]'"
              >
                {{ r.action === 'delete' ? t('admin-blocklist-act-delete') : r.action === 'flag' ? t('admin-blocklist-act-flag') : t('admin-blocklist-act-block') }}
              </span>
            </td>
            <td class="text-right">
              <AtomButton v-if="can('noro.admin.blocklist.edit')" variant="dark" icon="i-lucide-trash-2" class="!min-h-8 !px-2" @click="remove(r.id)" />
            </td>
          </tr>
        </tbody>
      </table>
    </section>

    <EmptyState
      v-else
      icon="i-lucide-shield-x"
      :title="t('admin-blocklist-empty-title')"
      :text="t('admin-blocklist-empty-text')"
    />
  </NoroShell>
</template>
