<script setup lang="ts">
interface Note {
  id: string
  author_label: string
  body: string
  created_at: string
}

const props = defineProps<{ userId: string }>()

const auth = useAuth()
const notify = useNotify()

const rows = ref<Note[]>([])
const body = ref('')
const busy = ref(false)

const load = async () => {
  rows.value = await auth.request<Note[]>(`/api/admin/users/${props.userId}/notes`)
}

async function add() {
  busy.value = true
  try {
    await auth.request(`/api/admin/users/${props.userId}/notes`, {
      method: 'POST',
      body: { body: body.value.trim() },
    })
    body.value = ''
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function remove(id: string) {
  try {
    await auth.request(`/api/admin/users/${props.userId}/notes/${id}`, { method: 'DELETE' })
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

onMounted(() => load())
</script>

<template>
  <section class="noro-panel p-5 space-y-4">
    <h3 class="text-lg font-black text-[var(--noro-text)]">Notes</h3>
    <p class="text-xs text-[var(--noro-muted)]">Admins only — the player never sees these.</p>

    <div class="grid gap-2">
      <textarea
        v-model="body"
        rows="3"
        class="noro-input w-full resize-y"
        placeholder="What happened"
      />
      <div>
        <AtomButton icon="i-lucide-plus" :loading="busy" :disabled="!body.trim()" @click="add">Add</AtomButton>
      </div>
    </div>

    <div v-if="rows.length" class="space-y-2">
      <div
        v-for="n in rows"
        :key="n.id"
        class="flex items-start justify-between gap-3 rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] p-3 text-xs"
      >
        <div>
          <div class="text-[var(--noro-text)]">{{ n.body }}</div>
          <div class="text-[10px] text-[var(--noro-muted)]">
            {{ n.author_label }} · {{ new Date(n.created_at).toLocaleString() }}
          </div>
        </div>
        <AtomButton variant="dark" icon="i-lucide-trash-2" class="!min-h-8 !px-2" @click="remove(n.id)" />
      </div>
    </div>
    <p v-else class="text-xs text-[var(--noro-muted)]">No notes yet.</p>
  </section>
</template>
