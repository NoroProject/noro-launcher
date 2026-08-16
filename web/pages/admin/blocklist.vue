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
  <NoroShell title="BLOCKLIST" subtitle="Files that must not be in a game folder">
    <UAlert
      class="mb-4"
      color="neutral"
      variant="subtle"
      icon="i-lucide-info"
      description="SHA1 is defeated by changing one byte, a name mask by renaming. Both catch the lazy, not the motivated — real coverage comes from the server side, where an agent verifies the mod set over its own channel. The list ships inside the signed manifest, so it cannot be swapped out on the client."
    />

    <section class="noro-panel mb-4 grid gap-4 p-4 md:grid-cols-[1fr_1fr_1fr_140px_auto]">
      <label class="block">
        <span class="noro-label mb-1.5 block">Name mask</span>
        <input v-model="pattern" class="noro-input w-full" placeholder="*xray*">
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">SHA1</span>
        <input v-model="sha1" class="noro-input w-full font-mono" placeholder="40 hex chars">
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">Reason</span>
        <input v-model="reason" class="noro-input w-full" placeholder="Known xray pack">
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">Action</span>
        <select v-model="action" class="noro-input w-full">
          <option value="delete">Delete</option>
          <option value="flag">Flag only</option>
          <option value="block_launch">Block launch</option>
        </select>
      </label>
      <div class="flex items-end">
        <AtomButton icon="i-lucide-plus" :loading="pending" @click="add">Add</AtomButton>
      </div>
    </section>

    <section v-if="rows.length" class="noro-panel overflow-x-auto">
      <table class="noro-table">
        <thead><tr><th>Mask</th><th>SHA1</th><th>Reason</th><th>Action</th><th /></tr></thead>
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
              >{{ r.action }}</span>
            </td>
            <td class="text-right">
              <AtomButton variant="dark" icon="i-lucide-trash-2" class="!min-h-8 !px-2" @click="remove(r.id)" />
            </td>
          </tr>
        </tbody>
      </table>
    </section>

    <EmptyState
      v-else
      icon="i-lucide-shield-x"
      title="Nothing blocked"
      text="Add a mask or a hash — the rules ship inside the signed manifest."
    />
  </NoroShell>
</template>
