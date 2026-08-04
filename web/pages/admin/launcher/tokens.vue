<script setup lang="ts">
import type { AdminTokenRow } from '~/types/api'

const auth = useAuth()
await auth.loadMe()

const { data: tokens, refresh, pending } = await useAsyncData('admin-tokens', () =>
  auth.request<AdminTokenRow[]>('/api/admin/tokens'), { default: () => [] }
)

const form = reactive({ name: '', permissions: 'noro.admin.*' })
const createdSecret = ref('')
const busy = ref<string | null>(null)
const showCreate = ref(false)

async function createToken() {
  busy.value = 'create'
  try {
    const response = await auth.request<{ id: string, token: string, permissions: string[] }>('/api/admin/tokens', {
      method: 'POST',
      body: {
        name: form.name,
        permissions: form.permissions.split('\n').map(s => s.trim()).filter(Boolean)
      }
    })
    createdSecret.value = response.token
    Object.assign(form, { name: '', permissions: 'noro.admin.*' })
    await refresh()
    showCreate.value = false
  } finally {
    busy.value = null
  }
}

async function revoke(id: string) {
  busy.value = id
  try {
    await auth.request(`/api/admin/tokens/${id}`, { method: 'DELETE' })
    await refresh()
  } finally {
    busy.value = null
  }
}
</script>

<template>
  <NoroShell title="TOKENS" subtitle="Tokens for CLI and CI">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
      <button type="button" class="noro-btn noro-btn-primary" @click="showCreate = true">
        <UIcon name="i-lucide-plus" class="size-5" />New token
      </button>
    </template>

    <UAlert
      v-if="createdSecret"
      class="mb-5"
      color="warning"
      variant="subtle"
      icon="i-lucide-key-round"
      title="Secret is shown once"
      :description="createdSecret"
    />

    <section class="noro-panel overflow-hidden">
      <table v-if="tokens?.length" class="noro-table">
        <thead><tr><th>Name</th><th>Permissions</th><th>Last used</th><th /></tr></thead>
        <tbody>
          <tr v-for="token in tokens" :key="token.id">
            <td class="font-semibold text-[var(--noro-text)]">{{ token.name }}</td>
            <td>{{ token.permissions.join(', ') }}</td>
            <td>{{ token.last_used_at || 'never' }}</td>
            <td class="text-right">
              <button class="noro-btn noro-btn-dark !min-h-8 !px-2" :disabled="busy === token.id" @click="revoke(token.id)"><UIcon name="i-lucide-trash-2" class="size-3.5" /></button>
            </td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-else icon="i-lucide-key-round" title="No tokens yet" text="Create a CLI token from the toolbar." />
    </section>

    <AtomModal v-model="showCreate" title="NEW TOKEN" subtitle="Secret will be shown once">
      <form class="grid gap-3" @submit.prevent="createToken">
        <label><span class="noro-label">Name</span><input v-model="form.name" class="noro-input" required></label>
        <label><span class="noro-label">Permissions, one per line</span><textarea v-model="form.permissions" class="noro-input min-h-28" /></label>
        <div class="flex justify-end gap-3 pt-2">
          <button type="button" class="noro-btn noro-btn-secondary" @click="showCreate = false">Cancel</button>
          <button :disabled="busy === 'create'" type="submit" class="noro-btn noro-btn-primary">
            <UIcon name="i-lucide-plus" class="size-5" />Create
          </button>
        </div>
      </form>
    </AtomModal>
  </NoroShell>
</template>
