<script setup lang="ts">
import type { Role, ServerRow } from '~/types/api'
import type { PermissionEntry } from '~/types/permissions'
import { toPermissionEntries } from '~/types/permissions'

const route = useRoute()
const auth = useAuth()
await auth.loadMe()

const id = computed(() => String(route.params.id))
const { data: roles, refresh } = await useAsyncData('admin-role-edit-list', () =>
  auth.request<Role[]>('/api/admin/roles'), { default: () => [] }
)
const { data: servers } = await useAsyncData('admin-role-servers', () =>
  auth.request<ServerRow[]>('/api/admin/servers'), { default: () => [] }
)
const role = computed(() => roles.value.find(item => item.id === id.value))
const permissions = computed(() =>
  toPermissionEntries(role.value?.permission_grants, role.value?.permissions || [])
)
const grants = usePermissionGrants(computed(() => `/api/admin/roles/${id.value}`))
const form = reactive({
  display_name: '',
  color: '#e85aa5',
  is_default: false,
  sort_order: 0,
  lp_group: '',
  icon: ''
})
const busy = ref<string | null>(null)

watchEffect(() => {
  if (!role.value) return
  Object.assign(form, {
    display_name: role.value.display_name,
    color: role.value.color || '#e85aa5',
    is_default: role.value.is_default,
    sort_order: role.value.sort_order || 0,
    lp_group: role.value.lp_group || '',
    icon: role.value.icon || ''
  })
})

async function run(name: string, action: () => Promise<void>) {
  busy.value = name
  try {
    await action()
    await refresh()
  } finally {
    busy.value = null
  }
}

async function save() {
  await run('save', () => auth.request(`/api/admin/roles/${id.value}`, { method: 'PUT', body: form }))
}

async function removeRole() {
  await run('delete', async () => {
    await auth.request(`/api/admin/roles/${id.value}`, { method: 'DELETE' })
    await navigateTo('/admin/roles')
  })
}

async function addPermission(entries: PermissionEntry[]) {
  await run('perm', () => grants.addMany(entries))
}

async function removePermission(entry: PermissionEntry) {
  await run(`perm-${entry.permission}`, () => grants.remove(entry))
}

async function movePermission(entry: PermissionEntry, serverId: string | null) {
  await run(`perm-${entry.permission}`, () => grants.move(entry, serverId))
}
</script>

<template>
  <NoroShell :title="role?.display_name || 'ROLE'" :subtitle="role?.name">
    <template #actions>
      <NuxtLink :to="'/admin/roles'" class="noro-btn noro-btn-ghost">
        <UIcon name="i-lucide-arrow-left" class="size-4" />
        Back
      </NuxtLink>
    </template>

    <EmptyState v-if="!role" icon="i-lucide-search-x" title="Role not found" />

    <div v-else class="grid gap-5 xl:grid-cols-[380px_1fr]">
      <form class="noro-panel grid gap-3 p-5" @submit.prevent="save">
        <label><span class="noro-label">Display name</span><input v-model="form.display_name" class="noro-input" required></label>
        <label><span class="noro-label">Color</span><input v-model="form.color" class="noro-input" type="color"></label>
        <label><span class="noro-label">Order</span><input v-model.number="form.sort_order" class="noro-input" type="number"></label>
        <label>
          <span class="noro-label">Icon</span>
          <input v-model="form.icon" class="noro-input" maxlength="8" placeholder="★">
          <span class="mt-2 block text-xs text-[var(--noro-muted)]">
            A single character shown next to the name. Unicode works everywhere &mdash;
            in the cabinet, in the launcher and in game chat.
          </span>
        </label>
        <label>
          <span class="noro-label">LuckPerms group</span>
          <input v-model="form.lp_group" class="noro-input" placeholder="vip">
          <span class="mt-2 block text-xs text-[var(--noro-muted)]">
            Links this role to a group in game. Leave empty if the role should not
            reach the server. Two roles cannot point at the same group.
          </span>
        </label>
        <UCheckbox v-model="form.is_default" label="Default role" />
        <div class="flex gap-2">
          <button type="submit" class="noro-btn noro-btn-primary" :disabled="busy === 'save'">
            <UIcon
              :name="busy === 'save' ? 'i-lucide-loader-circle' : 'i-lucide-save'"
              class="size-4"
              :class="busy === 'save' ? 'animate-spin' : ''"
            />
            Save
          </button>
          <UButton :loading="busy === 'delete'" icon="i-lucide-trash-2" color="error" variant="subtle" @click="removeRole">Delete</UButton>
        </div>
      </form>

      <AdminPermissionEditor
        title="Role permissions"
        subtitle="Everyone in this role inherits them. Pick a build to scope a permission to it."
        :entries="permissions"
        :servers="servers"
        :busy="busy"
        @add="addPermission"
        @remove="removePermission"
        @move="movePermission"
      />
    </div>
  </NoroShell>
</template>
