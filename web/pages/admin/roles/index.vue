<script setup lang="ts">
import type { Role } from '~/types/api'

const auth = useAuth()

const notify = useNotify()
await auth.loadMe()

const { data: roles, refresh, pending } = await useAsyncData('admin-roles', () =>
  auth.request<Role[]>('/api/admin/roles'), { default: () => [] }
)

const form = reactive({ name: '', display_name: '', color: '#e85aa5', is_default: false, sort_order: 0 })
const creating = ref(false)
const showCreate = ref(false)

async function createRole() {
  creating.value = true
  try {
    await auth.request('/api/admin/roles', { method: 'POST', body: form })
    Object.assign(form, { name: '', display_name: '', color: '#e85aa5', is_default: false, sort_order: 0 })
    await refresh()
    showCreate.value = false
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    creating.value = false
  }
}
</script>

<template>
  <NoroShell title="ROLES" subtitle="ACL through glob permissions">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
      <AtomButton variant="primary" icon="i-lucide-plus" @click="showCreate = true">New role</AtomButton>
    </template>

    <section class="noro-panel overflow-hidden">
      <table v-if="roles?.length" class="noro-table">
        <thead><tr><th>Role</th><th>Permissions</th><th>Default</th><th /></tr></thead>
        <tbody>
          <tr v-for="role in roles" :key="role.id">
            <td>
              <div class="flex items-center gap-2">
                <span class="size-2 rounded-full" :style="{ backgroundColor: role.color || 'var(--noro-magenta)' }" />
                <span class="font-semibold text-[var(--noro-text)]">{{ role.display_name }}</span>
              </div>
              <code class="text-xs text-[var(--noro-muted)]">{{ role.name }}</code>
            </td>
            <td>{{ role.permissions.length }}</td>
            <td>{{ role.is_default ? 'yes' : 'no' }}</td>
            <td class="text-right"><AtomButton variant="ghost" :to="`/admin/roles/${role.id}`" icon="i-lucide-settings" size="sm" /></td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-else icon="i-lucide-shield" title="No roles yet" text="Create the first role from the toolbar." />
    </section>

    <AtomModal v-model="showCreate" title="NEW ROLE" subtitle="Group permissions for launcher users">
      <form class="grid gap-3" @submit.prevent="createRole">
        <label><span class="noro-label">Name</span><input v-model="form.name" class="noro-input" required></label>
        <label><span class="noro-label">Display name</span><input v-model="form.display_name" class="noro-input" required></label>
        <label><span class="noro-label">Color</span><input v-model="form.color" class="noro-input" type="color"></label>
        <label><span class="noro-label">Order</span><input v-model.number="form.sort_order" class="noro-input" type="number"></label>
        <UCheckbox v-model="form.is_default" label="Default role" />
        <div class="flex justify-end gap-3 pt-2">
          <AtomButton variant="secondary" @click="showCreate = false">Cancel</AtomButton>
          <AtomButton variant="primary" icon="i-lucide-plus" :disabled="creating" type="submit">Create</AtomButton>
        </div>
      </form>
    </AtomModal>
  </NoroShell>
</template>
